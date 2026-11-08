//! clientkey — generate our own RSA-1024 handshake key pair and patch the
//! client's hardcoded server public key with it.
//!
//! Clean-room: we never read the original NCSoft key. We generate our own pair
//! (public exponent 65537, the value the client fixes in its `Rsa` constructor)
//! and write OUR modulus into a copy of the client. The server keeps the
//! matching private key. See Docs/Server/GameDaemon/08-client-handshake-key.md.
//!
//! Blob formats (little-endian, matching bloodlib::crypto):
//!   public  : [u32 n_len][n_le]                      (n_len = 128 for RSA-1024)
//!   private : [u32 n_len][n_le][u32 d_len][d_le]
//!
//! The public blob is byte-for-byte what the client's RsaPublic::Setup expects
//! at .rdata 0x13F6F80 (file offset 0xFF6380), total 132 bytes.

use bloodlib::crypto::RsaKey;
use rand::rngs::OsRng;
use rsa::traits::PrivateKeyParts;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use std::fs;
use std::process::ExitCode;

const CLIENT_KEY_OFFSET: u64 = 0xFF_6380;
const PUBLIC_BLOB_LEN: usize = 132; // 4-byte length + 128-byte modulus
const EXPECTED_MODULUS_LEN: usize = 128;

fn u32_le(n: usize) -> [u8; 4] {
    (n as u32).to_le_bytes()
}

/// `[u32 len][bytes]` little-endian field.
fn push_field(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&u32_le(bytes.len()));
    out.extend_from_slice(bytes);
}

fn gen(priv_path: &str, pub_path: &str) -> Result<(), String> {
    let mut rng = OsRng;
    // RSA-1024, default public exponent 65537 (matches the client).
    let key = RsaPrivateKey::new(&mut rng, 1024).map_err(|e| format!("keygen failed: {e}"))?;
    if key.e() != &rsa::BigUint::from(65537u32) {
        return Err(format!("public exponent is {}, must be 65537", key.e()));
    }

    let n_le = key.n().to_bytes_le();
    let d_le = key.d().to_bytes_le();
    if n_le.len() != EXPECTED_MODULUS_LEN {
        return Err(format!(
            "modulus is {} bytes, expected {EXPECTED_MODULUS_LEN}",
            n_le.len()
        ));
    }

    // Public blob = [u32 128][n_le]. Exactly 132 bytes.
    let mut pub_blob = Vec::with_capacity(PUBLIC_BLOB_LEN);
    push_field(&mut pub_blob, &n_le);
    if pub_blob.len() != PUBLIC_BLOB_LEN {
        return Err(format!(
            "public blob is {} bytes, expected {PUBLIC_BLOB_LEN}",
            pub_blob.len()
        ));
    }

    // Private blob = [u32 n_len][n_le][u32 d_len][d_le].
    let mut priv_blob = Vec::new();
    push_field(&mut priv_blob, &n_le);
    push_field(&mut priv_blob, &d_le);

    // Self-test: our pair must round-trip through the server's real handshake
    // code (RSA-1024 + OAEP/RIPEMD-128) before we trust it.
    verify_roundtrip(&pub_blob, &priv_blob)?;

    fs::write(pub_path, &pub_blob).map_err(|e| format!("write {pub_path}: {e}"))?;
    fs::write(priv_path, &priv_blob).map_err(|e| format!("write {priv_path}: {e}"))?;

    println!("generated RSA-1024 key pair (e=65537)");
    println!("  public  blob: {pub_path}  ({} bytes)", pub_blob.len());
    println!(
        "  private blob: {priv_path}  ({} bytes)  [keep OUT of the repo]",
        priv_blob.len()
    );
    println!("  self-test   : OAEP/RIPEMD-128 round-trip OK");
    Ok(())
}

/// Encrypt with the public blob, decrypt with the private blob, using the
/// server's own crypto. Proves the generated key works with the live handshake.
fn verify_roundtrip(pub_blob: &[u8], priv_blob: &[u8]) -> Result<(), String> {
    let pubk = RsaKey::from_public_blob(pub_blob).map_err(|e| format!("bad public blob: {e}"))?;
    let privk =
        RsaKey::from_private_blob(priv_blob).map_err(|e| format!("bad private blob: {e}"))?;
    let mut rng = OsRng;
    // A 16-byte session key is the real payload the client sends.
    let message = b"handshake-check0";
    let ct = pubk
        .encrypt(message, &mut rng)
        .map_err(|e| format!("encrypt: {e}"))?;
    let pt = privk.decrypt(&ct).map_err(|e| format!("decrypt: {e}"))?;
    if pt != message {
        return Err("round-trip mismatch".to_string());
    }
    Ok(())
}

fn patch(in_path: &str, out_path: &str, pub_path: &str, offset: u64) -> Result<(), String> {
    let blob = fs::read(pub_path).map_err(|e| format!("read {pub_path}: {e}"))?;
    if blob.len() != PUBLIC_BLOB_LEN {
        return Err(format!(
            "public blob is {} bytes, expected {PUBLIC_BLOB_LEN}",
            blob.len()
        ));
    }
    // Sanity: the blob we write must itself be a valid 1024-bit public key.
    RsaKey::from_public_blob(&blob).map_err(|e| format!("public blob is not a valid key: {e}"))?;

    let mut image = fs::read(in_path).map_err(|e| format!("read {in_path}: {e}"))?;
    let start = offset as usize;
    let end = start
        .checked_add(PUBLIC_BLOB_LEN)
        .ok_or("offset overflow")?;
    if end > image.len() {
        return Err(format!(
            "offset {offset:#x}+{PUBLIC_BLOB_LEN} exceeds file size {}",
            image.len()
        ));
    }

    // Safety check: the existing 4-byte length prefix must read 128. This
    // confirms we are at the key blob and not some unrelated bytes.
    let existing_len = u32::from_le_bytes(image[start..start + 4].try_into().unwrap());
    if existing_len as usize != EXPECTED_MODULUS_LEN {
        return Err(format!(
            "length prefix at {offset:#x} is {existing_len}, expected {EXPECTED_MODULUS_LEN}; wrong offset or file"
        ));
    }

    image[start..end].copy_from_slice(&blob);
    fs::write(out_path, &image).map_err(|e| format!("write {out_path}: {e}"))?;

    // Verify the write landed.
    let check = fs::read(out_path).map_err(|e| format!("reread {out_path}: {e}"))?;
    if check[start..end] != blob[..] {
        return Err("verification after write failed".to_string());
    }

    println!("patched client public key");
    println!("  in  : {in_path}");
    println!(
        "  out : {out_path}  ({} bytes, unchanged size)",
        check.len()
    );
    println!("  at  : {offset:#x} .. {end:#x}  ({PUBLIC_BLOB_LEN} bytes)");
    println!("  verify: read-back matches blob OK");
    Ok(())
}

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  \
         clientkey gen   --priv <file> --pub <file>\n  \
         clientkey patch --in <Client.exe> --out <Client.exe> --pub <file> [--offset 0xFF6380]"
    );
    ExitCode::from(2)
}

fn arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        return usage();
    };
    let rest = &args[1..];
    let result = match cmd.as_str() {
        "gen" => match (arg(rest, "--priv"), arg(rest, "--pub")) {
            (Some(p), Some(q)) => gen(&p, &q),
            _ => return usage(),
        },
        "patch" => {
            let (Some(i), Some(o), Some(p)) =
                (arg(rest, "--in"), arg(rest, "--out"), arg(rest, "--pub"))
            else {
                return usage();
            };
            let offset = match arg(rest, "--offset") {
                None => CLIENT_KEY_OFFSET,
                Some(s) => {
                    let s = s.trim_start_matches("0x");
                    match u64::from_str_radix(s, 16) {
                        Ok(v) => v,
                        Err(_) => return usage(),
                    }
                }
            };
            patch(&i, &o, &p, offset)
        }
        _ => return usage(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
