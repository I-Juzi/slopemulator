# 09 — Token login server (launcher path)

This spec is the server side of the STS **token login** (`LoginTokenStart` / `TokenKeyData`). It is the path the launcher uses: the client sends a token, not a password. It needs the client key patch of `08-token-key-patch.md` (already applied to `BNSR_2020_client/.../StsMsgCli64.dll`).

Status: spec from static analysis of `StsMsgCli64.dll` (image base `0x180000000`). **No real handshake captured.** Every ❓ below needs one real launcher login to confirm. The RSA and PRF are read from code, so treat this as a build-to spec, not a proven one. The SRP password path (`07`) is the verified, default path; build this only for launcher logins.

## Two different keys — do not confuse them

| Key | Lifetime | Role |
| --- | --- | --- |
| **Identity key** | Long-term; its public half is the patched blob in the DLL (`08`), private half = `~/Desktop/bns-handshake-key/sts_token_private.pem` | The server signs `ServerPublicKey` with it; the client verifies the signature. |
| **ServerPublicKey** | Per connection (ephemeral) | A fresh RSA key the server makes each session. The client encrypts the premaster and token to it. |

## Message flow

```
Client → LoginTokenStart   <ClientRand>
Server → reply 3111        <ServerRand> <ServerPublicKey> <ServerSignature>
   client verifies ServerSignature with the patched identity key, then:
Client → TokenKeyData      <PremasterSecret> <AuthnToken> <AuthProviderCode> <AppId>
   both derive the key block; RC4 both ways starts after this.
Client → LoginFinish ...   (as in 07, now over RC4)
```

## Number / RSA encoding (same as 07)

All RSA numbers (modulus, signature, ciphertext) and `ClientRand`/`ServerRand` cross the wire **Base64**. Inside, every big number uses the library's **word-LE** form: little-endian 32-bit words, high zero words trimmed (reuse `srp6::number_bytes` / `read_number`). RSA here is **raw modular exponentiation, no PKCS padding** (`CKeyRsa`, method `sub_18002E9E0`: import `sub_180031A90` word-LE → `out = in^exp mod n` → export `sub_1800316B0` word-LE).

| Field | Content |
| --- | --- |
| `ServerPublicKey` | The 268-byte CKeyRsa blob `[u32 type=4][u32 e=65537][u32 modlen=256][256-byte LE modulus]` of the ephemeral key, Base64. |
| `ServerSignature` | `RSA_raw( MD5(ServerPublicKey_bytes), d_identity, n_identity )`, word-LE, Base64. MD5 is the 16-byte digest of the exact `ServerPublicKey` wire bytes (the Base64-decoded blob). Client check: `RSA_raw(sig, e, n_identity)` first 16 bytes equal that MD5 (`sub_180001900`, hash `sub_18002DFF0` = MD5, classic IV). |
| `PremasterSecret` | `RSA_raw(premaster, e, n_ephemeral)` by the client; server decrypts with `d_ephemeral`. ❓ premaster length (TLS uses 48). |
| `AuthnToken` | `RSA_raw(token, e, n_ephemeral)`; server decrypts likewise. The token is the launcher-issued session token. |
| `AuthProviderCode`, `AppId` | Plain strings. |

## Key derivation (PRF `sub_1800013E0` → `sub_180001040`, suite id 4)

Suite 4 sizes (`unk_180056928`): MAC = 16, cipher key = **128**, IV = 0.

**PRF primitive** `sub_180001040(secret, randA, randB, out_len)` fills `out_len` bytes, 16 at a time. Block `i` (0-based):

```
label_i = (i+1) bytes, each = ascii 'A' + (i mod 26) + 1     # ❓ exact label: "B","CC","DDD",… per the table off_18006C000
inner_i = SHA256( label_i ‖ secret ‖ randA ‖ randB )          # sub_18002E800 = one-shot SHA-256
block_i = MD5( secret ‖ inner_i )                             # 16 bytes
```

**Two-stage expansion** (`sub_1800013E0`, suite 4):
1. `master = PRF(premaster, randA, randB, 48)`
2. `key_block = PRF(master, randB, randA, 288)`  — note the two randoms swap order between stages.

❓ Which random is `randA` (ClientRand vs ServerRand) in each stage is not yet pinned; follow the TLS convention (stage 1 = client‖server, stage 2 = server‖client) and confirm with a capture.

**Split the 288-byte key block, in order:** `client_mac[16] ‖ server_mac[16] ‖ client_key[128] ‖ server_key[128]` (no IV). The MAC keys are derived but **not used on the message path** — messages are RC4 only, no per-message MAC (send `sub_18001C300`, no MAC step).

**Cipher assignment** (from `sub_1800135A0`): the client makes its **send** cipher from `client_key` and its **receive** cipher from `server_key`. Therefore the **server** uses:
- RC4 **decrypt** (incoming) keyed with `client_key`,
- RC4 **encrypt** (outgoing) keyed with `server_key`.

RC4 key length is 128 bytes. One stream per direction, no reset — same whole-message rule as `07`.

## Server algorithm

1. **LoginTokenStart**: read `ClientRand`. Make an ephemeral RSA-2048 key (e=65537). Make `ServerRand` (❓ length; TLS uses 28). Reply 3111 with `ServerRand`, `ServerPublicKey` (ephemeral blob), `ServerSignature` (identity-signed MD5 of that blob).
2. **TokenKeyData**: decrypt `PremasterSecret` and `AuthnToken` with `d_ephemeral`. Validate the token against the launcher token store (single-use; see `tokens.rs`). Derive `master` then `key_block`; install RC4 (decrypt=client_key, encrypt=server_key). Reply (❓ reply id 3112) success.
3. From here the session is identical to `07`: `LoginFinish` → 3103, `RequestGameToken` → 3107, all over RC4.

## What to reuse from the SRP crate

- `srp6::number_bytes` / `read_number` for word-LE RSA I/O.
- `rc4::Rc4` (already whole-stream in `wire.rs`).
- `tokens::TokenAuthority` for single-use token validation.
- MD5 and the SSL3 PRF are **new** (not in the crate). Add an MD5 impl (or a cached crate) and the PRF above.

## Open items (need one real launcher login)

- ❓ PRF label bytes (the `'A'+…` table) and the randA/randB order per stage.
- ❓ `ServerRand` and premaster lengths.
- ❓ Reply id and body of the TokenKeyData response (3112?).
- ❓ `AuthProviderCode` values the client sends.
- ❓ Whether the launcher drives the client into token mode at all for a private server, or whether SRP (`07`) suffices. If SRP suffices, this whole path is optional.

## Security / policy

The identity private key (`sts_token_private.pem`) and the ephemeral keys are the server's own. No vendor private key is used. The patched client trusts our identity key by `08`. Keep `sts_token_private.pem` out of the repo.

## Implementation (Rust, `Program/Platform/Sts/src/token_login.rs`)

| Item | Where | Choice for each ❓ |
| --- | --- | --- |
| MD5, PRF, two-stage expansion | `md5`, `prf`, `KeyBlock::derive` | Labels `"B"`, `"CC"`, …; stage 1 = client‖server, stage 2 = server‖client |
| Raw RSA, blob, signature | `RawRsaKey` (`generate`, `from_pem`, `public_blob`, `sign_md5`, `decrypt`) | Ephemeral key per connection (`rsa` crate keygen) |
| State machine | `TokenSession` wraps a new 07 `password::Session` | After TokenKeyData the session continues as 07 (`AwaitFinish`) |
| ServerRand / premaster | `SERVER_RAND_LEN = 28`, `PREMASTER_LEN = 48` | Decrypted premaster is zero-padded to 48; longer is refused |
| AuthnToken | `launcher_token` | 36-byte GUID text (trailing NULs ignored) or ≤ 16 raw bytes |
| Token store | `TokenAuthority::issue_launcher` / `consume_launcher` | Launcher tokens carry a fixed owner tag; game-token `consume` refuses them and the reverse |
| Reply 3112 | `TOKEN_KEY_DATA_REPLY_ID`, `xml::token_key_data_reply` | Empty `<Reply/>` |
| RC4 start | `TokenSession::take_ciphers` | Installed after the 3112 reply is sent (same rule as 07) |
| XML element names | `token_login::xml` | Same as the field names in this spec |

Not wired into `listener.rs` or `wire.rs` yet: SRP stays the only live path. Wire it after one real launcher capture confirms the ❓ items. Tests: `tests/token_login.rs` (independent client transcribed from this spec).
