# 15 — AuthSrv: client login and token (original StServers)

Source: AuthSrv.exe 1.20.3350, IDA copy `ida/lanes/auth-1/AuthSrv.exe.i64`. No PDB.
Scope: the path that CliGate uses (LoginStart, KeyData, LoginFinish, RequestToken).
CliGate side: [11-cligate-protocol.md](11-cligate-protocol.md) §4, §9, §10, §11.

## 1. Dispatch

| Item | Fact | Source |
| --- | --- | --- |
| Command name table | id → name, protocol `Auth`. Same ids as CliGate §2. | sub_140110A30 |
| Ids in scope | 21 KeyData, 22 LoginStart, 23 LoginFinish, 30 RequestGameToken, 31 ConsumeGameToken, 32 RequestToken | sub_140110A30, table 0x1401AE1E0 |
| Parse | The request body is parsed into one request record (command id at offset 0) | sub_140110FC0, sub_1400D7710 |
| Parse error | text `AuMain: invalid message, parser line %u` | sub_1400D7130 |
| Pre-checks | sub_1400C94A0, sub_1400C9180, sub_1400DF820 run before the handler ❓ (not decoded) | sub_1400D7040 |
| Local or forward | If the request has a user and the user is not local (grid check), AuthSrv forwards the request to the owner with timeout 25 000 ms | sub_1400D68F0, sub_1400ECF00 |
| Handler switch | on the command id | sub_1400F3770 |

| Id | Command | Handler |
| --- | --- | --- |
| 21 | KeyData | sub_1400E5B30 |
| 22 | LoginStart | sub_1400E6910 |
| 23 | LoginFinish (also id 27) | sub_1400E6380 |
| 30 | RequestGameToken | sub_1400E7F50 |
| 31 | ConsumeGameToken | sub_1400E3D70 |
| 32 | RequestToken | sub_1400E8190 |

### 1.1 DB transaction record `CAuLoginTxn` (vtable 0x1401AA368)

| Offset | Field | Source |
| --- | --- | --- |
| +8 | reply id (the request id of the caller) | sub_1400E6910 |
| +12 | state (selects the reply handler) | sub_1400E6910 |
| +16 | command id | sub_1400E6910 |
| +24 | UserId (16 bytes) | sub_1400E6910 |
| +40 | SessionId (16 bytes) | sub_1400E6910 |

| Vtable slot | Use | Source |
| --- | --- | --- |
| +24 | dump: `ReplyId`, `ReplyType`, `Command`, `UserId`, `SessionId`, `SecAuthType` | sub_1400F69C0 |
| +32 | DB reply OK: `switch (state − 3)` | sub_1400F1FE0 |
| +40 | DB reply error: `switch (state)`; default sends the DB error unchanged to the caller | sub_1400F0B90 |

## 2. Login session record (0x210 bytes)

Made by sub_1400EE7C0 → sub_1400DD4E0. Kept in a list at service +224 (link offset at service +208).

| Offset | Field | Source |
| --- | --- | --- |
| +16 | SessionId: GUID. If GUID make fails: 16 random bytes. | sub_1400DD4E0 (sub_140103E20, sub_14010DBB0) |
| +32 | UserId (16 bytes) | sub_1400DD4E0 |
| +48, +64, +80 | copied from the request record (+304, +96, +112) ❓ (client address and app data) | sub_1400E6910 |
| +128 | SRP server object `CNpSrpServer` (wraps `CSrpServer`), random source sub_14010DBB0 | sub_1400DD4E0, sub_14012F880, sub_14012F230 |
| +136 | LoginName, max 65 bytes | sub_1400E6910 |
| +380 | validated flag (0 → log text `NOT VALIDATED`) | sub_1400E5B30 |
| +472 | Roles: u32 array | sub_1400DED50 |

Session delete: sub_1400F3F40 (unlink and free).

## 3. LoginStart (id 22)

### 3.1 Request (sub_1400E6910)

| Step | Fact |
| --- | --- |
| 1 | `<LoginName>` is required (request +1016). Else ErrBadParam 10 `missing login name`. |
| 2 | Make a login session (§2). Fail → code 4 `session create failed.` |
| 3 | Copy LoginName to session +136 (max 65 bytes). |
| 4 | If the user center (service +28) = own datacenter (sub_140157AF0): DB call `Exec` `AcctLoginInfoById` on logical DB `off_1402322A8` with schema version `v2016102001`, param = UserId (sub_140112220, sub_140112570, sub_1401124D0). Else: forward to the user datacenter (sub_14010F7D0). |
| 5 | Send with a `CAuLoginTxn`, state 7, command id from the request, SessionId = session +16, timeout 25 000 ms, route flag 287 (sub_140135670). |

### 3.2 DB reply, state 7 (sub_1400EDE00)

| Step | Fact |
| --- | --- |
| 1 | Find the session by txn SessionId. Not found → code 3101 `login start reply, session not found`. |
| 2 | Take the first row (sub_140112500). No row → `login start reply, missing row`; session deleted. |
| 3 | Parse the row (sub_1400D0410). Bad user ref → `AuthSrv: invalid user ref in row element`. Bad row → `login start reply, invalid loginname`; session deleted. |
| 4 | SRP start (sub_1400D1170): read row columns `Salt` and `Verifier` as binary (row vtable +224). If one is empty → code 3317, line 563, `no salt verifier`. Call SRP `Start(out keyData, loginName, salt, verifier)` (srp vtable +8). Fail → code 4, line 571, `start key exchange fail`. On error the session is deleted. |
| 5 | Store the row data on the session (sub_1400F57B0, sub_1400F5630) ❓. |
| 6 | Reply (line 4189): `<Reply><Session>GUID</Session><UserId>GUID</UserId><UserName>row UserName</UserName><UserCenter>row UserCenter</UserCenter><KeyData>…</KeyData></Reply>` (KeyData writer sub_1400F72D0). |

## 4. SRP-6a server (`CSrpServer`, vtable 0x1401BA5C8)

### 4.1 Group, hash and number format

| Item | Fact | Source |
| --- | --- | --- |
| N | 1024-bit. The value is equal to the RFC 5054 1024-bit group (first bytes `EEAF0AB9`). Stored as 32 big-endian u32 words at 0x1401BA540. | sub_14012DF20 |
| g | 2 | sub_14012DF20, sub_14012F450 |
| Hash H | SHA-256 (standard IV, standard digest byte order) | sub_14011E6B0, sub_14011ECA0 |
| H over many items | one SHA-256 over the items, in order, without separators | sub_14011E6B0 |
| Big number to bytes (`le(x)`) | little-endian, length = 4 × used u32 words (top zero words removed, so the length is always a multiple of 4) | sub_14017CD90 |
| Bytes to big number | little-endian | sub_14017CCC0 |
| Digest to number (`hnum(d)`) | reverse the order of the eight 4-byte groups of `d`, then read little-endian. (This is not the big-endian read of `d`.) | sub_14012DF20, sub_14012E960 |
| k | `hnum(H(le(N), le(g)))`. `le(N)` = 128 bytes, `le(g)` = `02 00 00 00`. | sub_14012DF20 |
| HNG | `H(le(N)) XOR H(le(g))`, 32 bytes, raw digest bytes | sub_14012DF20 (xmmword_1402955C0) |
| I | login name, ASCII `A`–`Z` changed to lower case, max 64 chars | sub_14012FA30, sub_140101830 |
| HI | `H(I)` (I without the end zero) | sub_14012F450 |

### 4.2 State and steps

| State (+8) | Meaning | Source |
| --- | --- | --- |
| 3 | new / done | sub_14012F230, sub_14012E960 |
| 1 | Start done, wait for client key data | sub_14012F450 |

**Start** (vtable +8, sub_14012F450; wrapper sub_14012FA30):

1. Fail (return 0) if I is empty, or the salt is not 8 bytes, or the verifier is not 128 bytes.
2. s = salt (8 bytes, little-endian). v = verifier (128 bytes, little-endian).
3. b = 32 random bytes (random source of §2), little-endian.
4. B = (k·v + g^b) mod N.
5. Out key data = `u32le len(le(s))` ‖ `le(s)` ‖ `u32le len(le(B))` ‖ `le(B)`.

**Step** (vtable +16, sub_14012E960). Input is the client key data.

1. Needs state 1. Else fail.
2. Read `u32le lenA` ‖ A (lenA bytes, little-endian) ‖ `u32le 32` ‖ M1 (32 bytes). All input must be used. A must be ≠ 0 and < N. Else fail.
3. u = `hnum(H(le(A), le(B)))`.
4. S = (A · v^u)^b mod N.
5. K = `interleave(le(S))` (§4.3), 64 bytes.
6. M = H(HNG, HI, le(s), le(A), le(B), K). Compare with M1. Not equal → fail.
7. M2 = H(le(A), M, K). Out key data = `u32le 32` ‖ M2. Complete = 1. State = 3.

**SessionKey** (vtable +24, sub_14012F020): K, 64 bytes.

### 4.3 interleave (sub_14012F030)

Input: `T = le(S)`, length n.

1. While n > 4 and `T[0]` ≠ 0: remove `T[0]`, n −= 1. (The code removes **non-zero** first bytes. Copy this exactly. The client uses the same code ❓.)
2. h = n / 2 (integer). `E[j] = T[n−1−2j]`, `F[j] = T[n−2−2j]`, for j = 0 … h−1.
3. G = H(E), Hf = H(F).
4. `K[2j] = G[j]`, `K[2j+1] = Hf[j]`, j = 0 … 31.

### 4.4 Rust test vector

No real client trace is available ❓. The Rust test makes its own vector: a
client side that uses §4.1–§4.3 (fixed b, fixed a, salt, password-derived x
❓ — the verifier formula of the account DB is not in AuthSrv). The server
must accept the client M1, return M2 that the client accepts, and both sides
must have the same K.

## 5. KeyData (id 21, sub_1400E5B30)

| Step | Fact |
| --- | --- |
| 1 | Find the session by request `<Session>` (request +840). Not found → ErrBadSession 3101 `login session not found.` |
| 2 | `<KeyData>` (request +992, length +1000) is required. Else ErrBadParam 10 `missing keydata.` |
| 3 | Each char must be Base64 (`A–Z a–z 0–9 + / =`). Else ErrBadParam 10 `invalid keydata.`; session deleted. |
| 4 | Decoded length bound 3·(len/4) = 0 → ErrBadParam 10 `invalid keydatalen.`; session deleted. |
| 5 | Base64 decode (sub_14011BD80). SRP Step (§4.2). |
| 6 | Step fails → session deleted, error ErrBadPasswd 11, module 6, line 7483, no body. Log `LOGIN-FAILED, <login name>`. |
| 7 | Step OK, but session +380 = 0 → session deleted, error ErrNeedEmailValidated 3320, module 6, line 7487. Log `NOT VALIDATED, <login name>`. |
| 8 | Step OK → reply code 0, module 6, line 7500: `<Reply>[<KeyData>Base64</KeyData>][<Complete/><SessionKey>Base64 K</SessionKey>]</Reply>`. KeyData only if out data is not empty. Complete and SessionKey only when the step completes. The session stays for LoginFinish. Log `LOGIN, <login name>` when complete. |

CliGate uses `SessionKey` for the RC4 filter (11 §4).
