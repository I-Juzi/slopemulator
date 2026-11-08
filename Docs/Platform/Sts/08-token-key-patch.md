# 08 — STS token-login key patch

This spec makes the STS **token login** (`LoginTokenStart` / `TokenKeyData`, see `07-client-login-wire.md`) work with a local server. It replaces the vendor verification key in the client library with our own key. It is the same method as the GameDaemon handshake key (`../Server/GameDaemon/08-client-handshake-key.md`), on a different file.

Use this only if the launcher forces the client into token mode. The password path (SRP) needs no client patch; prefer it. See `STATUS.md`.

## Where the key is

| Item | Value |
| --- | --- |
| File | `StsMsgCli64.dll` (2020 UE4 client; local copy `ida/sts-client/cli64/`) |
| Set by | `StsMsgCli` constructor `sub_1800194D0` |
| Form in file | one Base64 string in `.rdata`, decoded at construction into the verify-key slot (object `+160`) |
| String VA | `0x180059960` |
| **File offset** | **`0x58D60`** |
| String length | **361 bytes** (Base64 text, no terminator counted) |
| Decoded blob | 268 bytes |

## Blob format (the client's own public-key layout)

The decoded 268 bytes are a `Crypt::CKeyRsa` public key:

| Offset | Size | Field | Value in the vendor key |
| ---: | ---: | --- | --- |
| 0 | 4 | type/algo id (u32 LE) | 4 |
| 4 | 4 | public exponent e (u32 LE) | 65537 |
| 8 | 4 | modulus length in bytes (u32 LE) | 256 |
| 12 | 256 | modulus (2048-bit) | *(not copied — vendor key)* |

So it is a 2048-bit RSA public key, e = 65537. The modulus bytes are **not** recorded in this repo.

## How the client uses it (reply 3111 handler `sub_180004100` → verify `sub_180001900`)

```
digest   = MD5( ServerPublicKey bytes )          (sub_18002DFF0, classic MD5 IV)
recovered = RSA_public( ServerSignature )          (CKeyRsa, e=65537, embedded modulus)
accept iff  memcmp(recovered[0..16], digest) == 0  (sub_180001900)
```

- Hash H2 = **MD5**, 16-byte output.
- The signature is one RSA public operation; the client compares the first 16 bytes of the result to the MD5 digest. ❓ Whether padding (PKCS#1 v1.5) sits before those 16 bytes, or the layout is raw, is not yet confirmed byte-for-byte; the patch does not depend on it because we control both key and signer.

## The patch

1. Generate our own RSA-2048 key pair, e = 65537. Keep the private key **outside the repo**.
2. Build a 268-byte blob in the exact format above, with our modulus in the **same byte order** the vendor used (the tool detects the order from the original region and matches it).
3. Base64-encode. It is exactly 361 bytes, because the key size is identical, so the fixed-length copy (`memmove(..., 0x169)`) stays valid. No length immediate changes.
4. Overwrite 361 bytes at file offset `0x58D60`.

Result: the patched client checks `ServerSignature` with **our** public key. Our token-login server signs `MD5(ServerPublicKey)` with **our** private key, and the client accepts it.

## Server side (token path, future)

Our STS server, when it answers `LoginTokenStart` with reply 3111, must:
- send `ServerPublicKey` (the per-session RSA key the client encrypts the premaster/token with),
- send `ServerSignature = RSA_private( MD5(ServerPublicKey) )` using the patched key's private half,
- send `ServerRand`.

This is only needed for the token path. The SRP password path does not use any of it.

## Tooling

- Tool: `~/Desktop/bns-handshake-key/sts_key_patch.py` (sibling to the GameDaemon key tool). It generates the pair, writes the blob, patches a copy, and saves our key pair to that folder (outside the repo).
- The tool never writes vendor key bytes anywhere. It reads the original region only to confirm the offset and copy the modulus byte order.

## Ethics / policy

The replacement uses **our own** key, on the client file the operator owns, to make the operator's own client talk to the operator's own server. No vendor private key is derived, copied or used. This matches the client-patch policy already applied to GameGuard and the GameDaemon handshake key.
