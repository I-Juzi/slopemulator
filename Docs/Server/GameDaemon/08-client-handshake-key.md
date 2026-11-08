# Client handshake key (2020 client, build 381) — read-only finding

Goal: decide if our own server RSA key can be used. Source: passive IDA read of
`ida/delegate/client/Client_copy.i64` (full PDB). No key bytes were read or stored.

## Result: Case B — the client holds a hardcoded server public key

```
Client handshake (login AND game):
  AuthPacketProcessor::Encrypt ─┐
  GamePacketProcessor::Encrypt ─┴─► PPUtil::EncryptHandshake (0x60FB10)
        │
        ├─ RSA public setup from a FIXED blob in .rdata   → the server public key
        ├─ generates a fresh session key (key exchanger)
        └─ encrypts the session key under that RSA public key, sends it
  Server decrypts with its RSA PRIVATE key (which the client never holds).
```

| Fact | Value | Evidence |
| --- | --- | --- |
| Case | B (hardcoded, not received in handshake) | one `.rdata` constant, no network source |
| Key location | `.rdata`, address `0x13F6F80` (read-only, not executable) | segment perms r--; single xref |
| Key size | 132 bytes = 1024-bit modulus (128) + exponent (4) | `Setup(blob, 132, 1024)` in EncryptHandshake |
| Used by | login and game both | callers: `AuthPacketProcessor::Encrypt`, `GamePacketProcessor::Encrypt` |
| Number of keys | one, shared by both channels | the blob has a single xref (`0x60FBFB`) |
| Packed? | no — plain `.rdata` in Client.exe (the packed binary is BNSR.exe, not touched) | IDA reads it as a constant |
| Key bytes in repo | none | not read, by rule |

## Self-integrity check over the key region: none found

The only file-authentication machinery belongs to GameGuard, and it checks
GameGuard's own files, not the client image:

| Routine | Address | Target | Verdict |
| --- | --- | --- | --- |
| `CFileAuthClient::Auth2` + `MyCheckSumMappedFile` | `0x6FEF00` / `0x6FEE70` | files in the GameGuard directory, before `LoadLibraryA` | GameGuard only; callers `RunGGError`, `PreInitNPGameMonI`, `InitNPSC`. We do not run GameGuard. Boundary — not followed. |
| `PackSignature::CheckSignature` / `VerifySignatureKey` | `0x705BA0` | data-pack files by path (wstring) | content signing of data packs, not the running image |
| `CFvCrc32::Crc32File` | `0x6FC7B0` | a file by path | data-file CRC, not `.rdata` |

None of these checksums `.rdata` at `0x13F6F80`. So replacing the key does not
trip an in-client self-check.

## Conclusion

- Our own server key is accepted only by replacing the client's hardcoded public
  key (Case B). The client does not accept a server-supplied key.
- The replacement is ethical per the client-patch policy: our own public key,
  pointing the client at our server, no secret used, no protection defeated.
- Plan: generate our 1024-bit RSA pair. Put our PUBLIC key (132-byte layout:
  128-byte modulus + 4-byte exponent) at `0x13F6F80` in a COPY of Client.exe.
  Our server keeps the PRIVATE key. Original client stays read-only.

## Open (❓)
- Resolved: the user launches **Client.exe** directly (not via BNSR.exe). Client.exe
  is a normal PE (plain `.rdata`), so the patched key loads as written. No packed
  loader is in the path.
- ❓ Exact 132-byte field layout on the wire (modulus endianness, exponent width)
  before we build our key blob. Verify at patch time, still without reading the
  original bytes.

## Exact key blob format (from RsaPublic::Setup 0x611E50, Rsa ctor 0x611840, BigInteger::Set 0x60BC50)

```
blob @ .rdata 0x13F6F80  (file offset 0xFF6380), total 132 bytes
  [0x00:0x04]  u32 little-endian = modulus length in bytes = 0x00000080 (128)
  [0x04:0x84]  modulus, 128 bytes, LITTLE-ENDIAN (least-significant byte first)
  exponent    NOT in blob. Fixed in client = 65537 (0x10001).
```

Cipher (client side): RSA-1024, OAEP padding, RIPEMD-128 hash (Rsa ctor sets
`OaepEncoder` + `Ripemd128` vftables; exponent member = 65537). So our server must
RSA-1024 **OAEP(RIPEMD-128)** decrypt the client block with our private key.

To patch: generate our RSA-1024 pair with e=65537, write our 128-byte modulus
little-endian after the 4-byte length prefix, at file offset 0xFF6380 in a COPY of
Client.exe. No original bytes are read. Our private key stays OUT of the repo.
❓ OAEP label and MGF1 hash: assume MGF1 over RIPEMD-128, empty label; verify at integration.

## Done (2026-11-08)

The RSA handshake already exists in `bloodlib::crypto` (RSA-1024 + OAEP/RIPEMD-128
+ AES-128, from the LobbyGate phase). Added the key tool and ran the patch:

| Item | Location | Note |
| --- | --- | --- |
| Key tool | `Program/Tools/ClientKey` (`clientkey gen` / `clientkey patch`) | generates our RSA-1024 pair (e=65537), self-tests a round-trip through bloodlib's OAEP, patches a copy |
| Our private key | `../server-keys/handshake_private.blob` | OUTSIDE the repo. Server loads it: `lobbygate --handshake-key <path>` |
| Patched client | `../Client_patched.exe` | copy; original `Client.exe` untouched |
| Patch extent | 128 bytes changed at `0xFF6380+4` (modulus only); length prefix and file size unchanged | verified with `cmp` |

Our public modulus is now in the client; the server holds the matching private
key. No NCSoft key was read. ❓ Live client+server handshake not yet tested end to end.
