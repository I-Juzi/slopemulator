# 02 — Client handshake and session crypto

Target build: LobbyGate `0.0.381.572` (2020). Reference build: `0.0.348.527` (2019, with PDB).
Status: **draft 2** — byte-level crypto rules added in §8–§11. Sol drafted and reviewed them in IDA. The main session re-read the OAEP label bytes, `Dh::setupFactors` and `Dh::getValue` in 381. Other facts have no main-session check.

## 1. Summary

```
client                                                     LobbyGate
  │  HelloReq  cmd=1 "hhb"  {cmd, version=58, blob}           │   (client→server packets: RSA-1024 with the
  │  ─────────────────────────────────────────────────────▶  │    server's built-in private key, code 0x4)
  │                                                           │   blob = client RSA-512 public key + DH(g, p, A)
  │                                                           │   server: B = g^x mod p, x = 64-bit random   
  │                                                           │   session key K = first 16 bytes of A^x mod p
  │  reply  cmd=2 "hb"  {cmd, blob = DH reply value}          │   (server→client: RSA-512 with the client's
  │  ◀─────────────────────────────────────────────────────  │    public key, code 0x5)
  │                                                           │   then: current crypt := session crypt
  │  all later packets: AES-128-ECB with K (codes 0x8–0xB)    │
  │  ◀════════════════════════════════════════════════════▶  │
```

## 2. Sources

| Item | 2020 (381) | 2019 (348) |
| --- | --- | --- |
| Hello handler | `HandleLobbyGate_HelloReq` `0x140015200` | same name |
| Handshake parameter step | `0x140019E90` | `Session::SetHandshakeParameter` |
| Crypt factory | `PacketCrypt::Create` `0x14005EE80` (renamed by hand) | `PacketCrypt::Create` |
| Handshake crypt | `HandshakePacketCrypt::*` (vtable match) | same |
| Session crypt | `SessionPacketCrypt::*` (vtable match) | same |
| Built-in server RSA key blob | `0x140077290`, 264 bytes | `0x14006C370`, 264 bytes — **identical** (SHA-256 equal) |

## 3. Crypt modes of a session

`Session::_cryptMode` (2019 field names):

| Mode | Meaning | Set by |
| --- | --- | --- |
| 2 | Waiting for HelloReq; current crypt is null until the first encrypted frame creates the handshake crypt | session accept (§11) |
| 3 | Session key set; reply still to send | `SetHandshakeParameter` success |
| 4 | Handshake failed | HelloReq decode/version failure from mode 2 or 3; also `SetEncryptParam(PARAM_HANDSHAKE_FAIL)` |
| 5 | Done; current crypt = session crypt; handshake crypt freed | `Session::completeHandshake`, called right after the reply is sent when mode = 3 |

## 4. HelloReq (client → server, command 1)

Handled before login (`NotAuthed` table, command 1).

| Field | Format | Rule | Status |
| --- | --- | --- | --- |
| `cmd` | `h` (i16) | = 1 | ✅ |
| `version` | `h` (i16) | must be **58** in both 348 and 381; else log `Invalid version %d (Current %d)` and fail | ✅ |
| `blob` | `b` (length-prefixed bytes, max 4000) | must be non-empty; else `Invalid handshake data` → `FailHandshake` | ✅; prefix is `i16` (03 §5) |

Failure → no reply; the session goes to mode 4 and normal closure starts. Pending sends can delay `closesocket` (§11).

### 4.1 Blob layout (input to `HandshakePacketCrypt::SetKey`)

```
u32  L                       (little-endian)
u8   rsa_pub[L]              client RSA public key → RsaPublic::Setup(blob, L+4, 512 bits)
---- rest → Dh::Process (state 0 → Dh::setupFactors):
u32  n1 ; u8 g[n1]           DH generator        (0 < n1 ≤ 512)
u32  n2 ; u8 p[n2]           DH prime            (0 < n2 ≤ (2·bits+7)/8 = 32)
u32  n3 ; u8 A[n3]           client DH public value (n3 > 0)
```

All integers are little-endian (§8). `setupFactors` also fails when `p ≤ g` (`0x140067850`; `BigInteger::absCompare` `0x1400696A0` returns true when its first value is less than or equal to its second). ✅ main-session check.

| Check | Error |
| --- | --- |
| total size < 4, or < L+4 | 5 (`FAILED…` invalid arg) |
| handshake crypt already keyed | 4 `ALREADY_INITIALIZED` |
| RSA setup or DH setup fails | 6 `FAILED_SETUP_CRYPTO` |
| DH process fails | 11 `FAILED_KEYGEN` |

DH has **no built-in parameters**: the client supplies p, g and A. `Dh::Setup(128)` sets the key size to 128 bits. The server private exponent x has `max(64, bits/2)` = **64 bits**, with the high and low bits set (§8).

### 4.2 Server reply (server → client, command 2)

`Packet::Compose("hb", 2, …)`: `cmd = 2`, then the blob from `GenerateKey` — `B = g^x mod p` in minimal-length little-endian form. It has no padding. 32 bytes is the buffer capacity (§8). The reply is encrypted with the **handshake** crypt (client's RSA-512 key, code 0x5), then the session switches to the session crypt.

### 4.3 Session key

`Dh::GetKey`: `S = ExpMod(A, x, p)`; output length = `(bits+7)/8` = **16 bytes**. The first `min(len(S),16)` bytes of S are copied; if S is shorter, its bytes repeat cyclically until 16 bytes. S is little-endian (§8).

## 5. Handshake-phase packet crypt (`HandshakePacketCrypt`)

Members: `+16` RsaPublic (client key), `+1448` RsaPrivate (built-in server key), `+3408` Dh.

| Direction | Key | Header code (top 4 bits of u16) |
| --- | --- | --- |
| client → server | server RSA-1024 private (`Decrypt` accepts only code 0x4 `INITIAL_KEY`) | `0x4` |
| server → client (mode 0) | client RSA-512 public | `0x5` `CLI_HANDSHAKE_KEY` |
| server → client (mode 1) | server RSA-1024 private | `0x4` |

Wire frame: `u16 hdr` + ciphertext. `hdr = (code << 12) | ((cipher_len >> 2) & 0xFFF)`; total = `4 × (hdr & 0xFFF) + 2`. Wrong code → error 7 `FAILED_INVALID_CODE`; short input → 10 `FAILED_NOT_COMPLETE_PACKET`.

RSA padding: custom OAEP with RIPEMD-128. See §9.

## 6. Session packet crypt (`SessionPacketCrypt`)

- Two AES-128 instances (slots 0 and 1) — confirmed in both builds (`PacketCrypt::Create`, `SESSION_CRYPT`).
- `SetKey(K)` keys slot 0 and sets both slot indexes to 0.
- **AES-128-ECB**, 16-byte blocks; the last partial block is zero-padded; ciphertext length = round up to 16 (`Aes128::EncSize`, `Aes128Table_Encrypt`). ✅
- Header: `hdr = (code << 12) | ((cipher_len >> 2) & 0xFFF)`, code:

| Code | Name | When |
| --- | --- | --- |
| `0x8` | `SESSION_KEY_1` | slot 0 |
| `0x9` | `SESSION_KEY_2` | slot 1 |
| `0xA` | `SESSION_KEY_1_AND_3RD_PARTY` | slot 0, `security3rd` flag |
| `0xB` | `SESSION_KEY_2_AND_3RD_PARTY` | slot 1, `security3rd` flag |

- Key rotation: `GenerateKey` fills 16 random bytes (MSVC `rand()`), keys the *other* slot, and sets a "change pending" flag. The next `Encrypt` uses the old slot, then flips to the new slot. **381 has no live trigger and no packet that carries the key (§10). Do not add rotation in Rust.**
- Rust: use a CSPRNG for x. The value is random, so this keeps wire compatibility. Keep the 64-bit size and the set high and low bits.

## 7. Built-in server RSA key

- 264-byte blob: `[u32 128][128 bytes][u32 128][128 bytes]` — modulus, then private exponent, both little-endian (§8).
- The same key is in 348 and 381. ✅ The operator's 2020 `Client.exe` (PE32, PE timestamp 2020-06-04) contains the same 128-byte modulus in little-endian form, once, at file offset 16737156 in `.rdata`. A compatible server must use **this** key. Not yet shown: that the client uses this copy for its first packet (Phase 4 test). The operator's 2017 `Client (2).exe` (PE timestamp 2017-07-12) also contains it once, at file offset 11282180.
- **Rust rule:** the key is NCSoft's secret material. It must not be committed to the repo or compiled into the binary. The server loads it at start from an operator-supplied file (`--handshake-key <path>`). A small helper may extract it from the operator's own `LobbyGate.exe` (381: RVA `0x77290`). Without the file the server refuses to start.

## 8. Integer order and key layouts

✅ **RSA and DH integers use unsigned little-endian bytes.** Length fields use little-endian `u32`. The code copies bytes into low-order integer limbs without reversal. Sources: 381 `RsaPrivate::Setup` **`0x140066B50`**, `RsaPublic::Setup` **`0x1400668E0`**, `Dh::setupFactors` **`0x140067850`**, and `Dh::GetKey` **`0x140067660`**. These are also the 348 function names.

**264-byte private-key blob — layout only:**

| Offset | Size | Field |
| --- | ---: | --- |
| `0x000` | 4 | Modulus length: 128 |
| `0x004` | 128 | Modulus `n`, little-endian |
| `0x084` | 4 | Private-exponent length: 128 |
| `0x088` | 128 | Private exponent `d`, little-endian |

✅ The first integer becomes the modulus at RSA object offset `+0x390`. The second becomes the private exponent at `+0x598`. Both private operations use that exponent. Sources: **`0x140066B50`**, **`0x140066CF0`**, **`0x140066D50`**.

✅ The public-key blob contains `[u32 modulus_length][modulus_le]`. It contains **no exponent field**. `Rsa::Rsa` sets `e = 65537` at **`0x1400661A0`**.

✅ **The DH input order is `g, p, A`, not `p, g, A`:**

`[u32 g_length][g_le][u32 p_length][p_le][u32 A_length][A_le]`

The exponentiation uses the first field as its base and the second as its modulus. Sources: `Dh::setupFactors` **`0x140067850`**, `Dh::getValue` **`0x140067A10`**.

✅ `Dh::Setup(128)` selects a 16-byte session key. It does **not** select a 128-bit private exponent. `getValue` generates `max(64, bits/2)` exponent bits: **64 bits here**, with the high bit and low bit set. It uses the thread-local AES-CTR PRNG. Sources: **`0x14005FCF0`**, **`0x140067A10`**, `Prng::setSeed` **`0x14006A0A0`**.

✅ The reply blob is `B = g^x mod p`, in minimal-length little-endian form. It has **no internal length prefix or fixed 32-byte padding**. The 32-byte value in the caller is buffer capacity. Sources: **`0x140067A10`**, `HandshakePacketCrypt::GenerateKey` **`0x14005FDF0`**.

✅ `S = A^x mod p` also uses little-endian bytes. Use its first 16 bytes. If its representation is shorter, repeat its bytes cyclically to reach 16 bytes. Source: `Dh::GetKey` **`0x140067660`**.

## 9. RSA operation and OAEP variant

✅ This is **custom OAEP**, not standard RSAES-OAEP.

| Parameter | Exact rule | 381 source; 348 name |
| --- | --- | --- |
| Hash | Modified RIPEMD-128; 16-byte digest. Whole 64-byte blocks are standard. See the tail rule below this table. | `0x1400661A0`, `0x140066EC0`, `Ripemd128::Digest` `0x14006F600`; `Rsa::Rsa`, `Ripemd128::Size` |
| Label | Exactly 16 **zero** bytes; not an empty label. The bytes at `0x1400A6808` are in the part of `.data` that has no file data, so the loader sets them to zero. IDA shows `0xFF` for such bytes, which caused the earlier wrong value. IDA lists one reference (a read at `0x140069AED`) and no references from `0x1400A6608` up to it. Emulation of encode gives label hash `74c333697c413d8f390cf12124478cc3`, which equals RIPEMD-128 of 16 zero bytes. ❓ A computed-address write is not excluded. | `0x1400699D0`; `OaepEncoder::encode` |
| MGF | Concatenate `RIPEMD128(input ‖ u32_le(counter))`, starting at counter 0; truncate | `0x140069750`; `MaskGenerator::generate` |
| Seed | 16 bytes from the thread-local PRNG | `0x1400699D0`; `OaepEncoder::encode` |
| Capacity | `k − 34` message bytes per RSA block | `0x140069930`; `OaepEncoder::Setup` |

✅ **Tail rule (381).** The tail loop in `Ripemd128::Digest` stores each tail byte with `mov dword` at **`0x14006F92F`**, not with OR. Each store replaces the whole 32-bit word. Only the last byte written to each word stays. Padding and the bit length are standard. Equivalent form: in the bytes after the last whole 64-byte block, set a byte to zero unless its offset is 3 mod 4 or it is the last message byte. Then apply standard RIPEMD-128. This changes every digest with a non-empty tail, and so every MGF output. For a 16-byte MGF input, the output repeats every 16 bytes: the counter's three low bytes are overwritten by its high byte, which is zero for counters below 2^24. Evidence: emulation of `0x14006F600` and `0x140069750` (6 of 6 vectors match this rule; 5 of 6 differ from the standard). The 348 Hex-Rays output shows the same assignment in `Ripemd128::Digest` `0x14005E890` (not emulated). The 2020 client has the same rule (next paragraph).

✅ **Client 2020 (passive read, `Client.exe`, PE timestamp 0x5ED8E1F4).** `Ripemd128` digest `0x616F00` (RTTI `.?AVRipemd128@@`, vtable `0x1870E14`) has the same overwrite: `mov [esp+ecx*4+0x20], ebx` at `0x61705D`. `OaepEncoder` (vtable `0x1870E00`) hashes 16 bytes at `0x1A4BE00` (`push 0x10; push 0x1A4BE00` at `0x615D8F`). These bytes are zero in the file, and this is the only direct reference. A computed write is not excluded ❓.

✅ Before integer conversion:

- `DB = H(label) ‖ zero_padding ‖ 01 ‖ message`.
- `maskedDB = DB XOR MGF(seed, k−17)`.
- `maskedSeed = seed XOR MGF(maskedDB, 16)`.
- `EM = 00 ‖ maskedSeed ‖ maskedDB`.

The encoder reverses the complete `EM` byte sequence for little-endian integer use. RSA ciphertext blocks also use **little-endian**, fixed-width integers. Sources: **`0x1400699D0`**, `Rsa::encrypt` **`0x1400662A0`**, `Rsa::decrypt` **`0x140066580`**.

**Worked layout example:** RSA-512, `k = 64`, placeholder message `MESSAGE[20]`.

| Conceptual `EM` offsets | Contents |
| --- | --- |
| `0` | `00` |
| `1..16` | `maskedSeed[16]` |
| `17..63` | `maskedDB[47]` |

Here, `DB[47] = H(LABEL)[16] ‖ ZERO_PAD[10] ‖ 01 ‖ MESSAGE[20]`. Reverse `EM[64]` before importing it as a little-endian integer. The block contains no real key values.

✅ **Operations and block sizes:**

| Direction | RSA operation | Message bytes per block → ciphertext bytes |
| --- | --- | --- |
| Client → server, code `0x4` | Peer forms `c = EM^65537 mod n_server`; server applies `c^d_server mod n_server`, then OAEP decode | Up to **94 → 128** |
| Server → client, code `0x5` | Server applies `EM^65537 mod n_client`; peer must apply its private exponent, then OAEP decode | Up to **30 → 64** |
| Server → client, failure mode 1, code `0x4` | Server applies `EM^d_server mod n_server`; inverse uses public exponent 65537 | Up to **94 → 128** |

Sources: **`0x14005FF00`**, **`0x140060040`**, **`0x140066A10`**, **`0x140066CF0`**, **`0x140066D50`**, **`0x1400661A0`**.

✅ Split long plaintext into capacity-sized chunks. Encode each chunk separately. The last chunk retains its actual length. Ciphertext input must contain whole RSA blocks. Sources: **`0x1400662A0`**, **`0x140066580`**.

✅ **Decode is permissive:** it does not validate the leading `00` or the recovered label hash. After unmasking, it skips zeros after the 16-byte hash area and requires a `01` delimiter. Failure returns false; `Rsa::decrypt` sets error **17**, and handshake decryption returns error **13**. No fallback plaintext is accepted. Sources: `OaepEncoder::decode` **`0x140069D30`**, **`0x140066580`**, **`0x140060040`**.

Rust: `rsa::Oaep` does not fit, because of the little-endian MGF counter, the tail rule, the zero label and the permissive decode. Write a custom encoder and decoder around `ripemd::Ripemd128`. Use raw RSA or `BigUint::modpow`, and convert fixed-width blocks as little-endian.

## 10. Session-key rotation

✅ **381 has no live session-key rotation trigger or key-carrying packet.** The session `GenerateKey` method remains in its vtable, but the inspected runtime crypto-generation calls serve handshake setup and broadcast setup, not session rotation. Sources: session method **`0x14005F240`**, vtable entry **`0x140099BA0`**, handshake caller **`0x140019E90`**, broadcast caller **`0x1400334E0`**. Do not invent a rotation command or timer.

✅ The unused rotation mechanism has these rules:

| Step | Rule | 381 source |
| --- | --- | --- |
| Initial key | Key slot 0; transmit and receive indexes both become 0 | `0x14005F1E0` |
| Generate | Require equal indexes and no pending change; fill the other slot with 16 bytes, each `rand() & 0xFF`; set pending | `0x14005F240` |
| Next successful send | Encrypt with the old slot and old code; then flip the transmit index and clear pending | `0x14005F3B0` |
| Later sends | Slot 0 uses `0x8`; slot 1 uses `0x9` | `0x14005F3B0` |
| Receive transition | Retain the old receive slot until the received code changes; then clear the old key and use the transmit slot | `0x14005F4D0` |

348 names: `SessionPacketCrypt::{SetKey,GenerateKey,Encrypt,Decrypt}`.

✅ `rand()` comes from the imported CRT function at **`0x1400737C0`**. I/O workers seed it with `time64(NULL) + worker_id` at **`0x140043FA0`** (`IoWorker::run`). Crypto initialization also calls `srand(time64(NULL))` at **`0x140066DB0`**. This is not an OS random source.

## 11. Mode 2 and handshake failure

✅ Mode 2 enters through **session initialization during socket acceptance**. The accept path passes `Port +0x14` as the mode argument; initialization writes that argument to `Session +0x100`. Current-crypt pointers remain null until the first encrypted frame arrives.

| 381 address | Behaviour | Correct 348 name |
| --- | --- | --- |
| `0x140044960` | Read port mode and pass it into acceptance | `PortListener::accept` |
| `0x140018780` | Forward the mode to initialization | `Session::Accept` |
| `0x140018220` | Store mode; clear crypt pointers | `Session::initialize` |
| `0x1400330C0` | In mode 2, create handshake crypt on first encrypted frame | `ClientPacketBufferHandler::ToPacket` |

The IDB has incorrect automatic names on the middle two functions.

✅ `Port::Initialize` stores 2 at `Port +0x14` for the client port (kind 0, store at `0x140035CD4`). See 05 §8.5.

✅ A malformed HelloReq, wrong version, empty blob, or failed parameter setup sets mode 4 from mode 2/3 and returns false. It sends **no failure reply**. Source: `HandleLobbyGate_HelloReq` **`0x140015200`**.

✅ The failure propagates through `Session::handlePacket` **`0x140019600`**, `Session::HandleEvent` **`0x1400191A0`**, and the worker failure path **`0x140043FA0` → `0x140044230`**. That path starts normal socket closure through **`0x140018CC0`** (348 `Session::close`). Closure can wait for pending sends. It is **not always an immediate `closesocket`**.

✅ RSA decode failure takes the same receive-error close path before HelloReq runs. It need not set crypt mode 4. Sources: **`0x140060040`**, **`0x1400330C0`**.

## 12. Stateful session packet rules (381 emulator evidence)

Source: `tools/emu/SESSION-NOTES.md`, `SESSION-DIFFERENCES.json`, and the saved
`crates/proto/tests/vectors/emu_381.json` cases. Build: `0.0.381.572`.
Original methods: SetKey `0x14005F1E0`, GenerateKey `0x14005F240`,
Encrypt `0x14005F3B0`, Decrypt `0x14005F4D0`.

| Operation | Rule | Error |
| --- | --- | --- |
| SetKey | Set slot 0; select send = receive = 0; clear pending. | — |
| Encrypt | Round plaintext length up to 16. The padded ciphertext limit is 4000 bytes. The 2-byte header is outside this limit. | 8 if too large |
| Encrypt | Output capacity must hold the padded ciphertext and the 2-byte header. | 9 if too small |
| Encrypt | Use the current send slot. Add 2 to its tag when the third-party flag is set. | 3 if no key |
| Pending send | After a successful send, select the other send slot and clear pending. A rejected send must not change state. | — |
| Receive | Accept tags 8–11 only. Remove the third-party tag bit for slot selection. | 7 for another tag |
| Receive | Accept the receive slot. Accept a different slot only when it is the current send slot. Reject an inactive slot even if it has a key. | 7 |
| Receive transition | Keep the old receive key until the new tag arrives. Then clear the old key and select the send slot for receive. | — |
| Receive | Require the complete header and declared ciphertext. Require whole AES blocks. | 10 if incomplete |
| Prepare change | Require equal send/receive indexes and no pending change. Set the other key and mark pending. | — |

The saved vectors cover the 4000-byte boundary, both tag flags, rejected sends,
and the old/new send and receive transitions. A Rust packet type must own these
indexes and the pending flag. The fixed-slot `SessionCrypt` API may remain.
Expose explicit key-change preparation for tests and callers. Do not add a live
rotation command, timer, or key generator to the server (see §10).

## 13. Open items

1. Check §8–§9 against a test vector from the original code (Phase 2 emulation).
