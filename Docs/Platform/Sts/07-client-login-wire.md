# 07 — Client login wire format

## Source

Source C64: `StsMsgCli64.dll` from the 2020 UE4 client (BNSR), file date 2020-01-14, 488 960 bytes.
Local copy: `ida/sts-client/cli64/StsMsgCli64.dll`. Image base `0x180000000`. No PDB (own PDB path: `StsMsgCli64.pdb`, GUID e977af1a847f44579f9cbef2c054c4af, not available).
Source file names in the DLL: `StsMsgCli.cpp`, `Services/Srp/Srp.cpp`, `Services/SrpWrapper/SrpWrapper.cpp`, `Services/Crypt/CptRc4.cpp`, `Services/Base64/Base64.cpp`.
❓ The 32-bit `StsMsgCli32.dll` of the 2020 UE3 client is not checked yet. Compare it when the operator has it.

This closes the main gaps of `06-password-login.md`: the request bodies, the reply fields and the password-proof exchange.
Framing (`POST /Auth/<Command> STS/1.0`, headers `l` and `s`) is the same library code as in `01-transport.md`.

## Request bodies

All builders use `sub_18002F670(<enum>)` for the request line, then one `<Request>` element.
The enum maps to the command name through `0x18002EF10` (protocol `Auth`).

| Enum | Command | Body | Builder | Precondition (else library result) |
| ---: | --- | --- | --- | --- |
| 25 | `Auth/LoginStart` | `<Request><LoginName>name</LoginName></Request>` | `0x18001C780` | Login name and password set, SRP object present (else 4) |
| 22 | `Auth/KeyData` | `<Request><KeyData>base64</KeyData></Request>` | `0x18001C8C0` | Pending SRP output length ≠ 0 (else 8) |
| 24 | `Auth/LoginFinish` | `<Request><Language>n</Language></Request>` | `0x18001C9C0` | — |
| 35 | `Auth/RequestGameToken` | `<Request><GameCode>code</GameCode><AccountAlias>alias</AccountAlias></Request>` | `0x18001D2D0` | Alias set (else 4). GameCode comes from library init (`a1+96`). |
| 46 | `Auth/RequestToken` | `<Request><AppId>id</AppId></Request>` | `0x18001D410` | AppId set (else 4) |
| 28 | `Auth/LogoutMyClient` | `<Request/>` (empty element) | `0x18001D200` | — |
| 17 | `Auth/GetMyUserInfo` | `<Request/>` | `0x18001D080` | — |
| 49 | `Auth/VerifyOtp` ❓ name truncated in table | `<Request><OtpValue>..</OtpValue><ForceVerify>n</ForceVerify></Request>` | `0x18001CAB0` | — |

**The password is never sent.** `LoginStart` sends only the login name. The password goes only into the local SRP object.
`AccountAlias` and `AppId` use the wide-string writer `sub_180037D70`. ❓ Their text encoding on the wire (UTF-8 conversion) is not checked.
`GameCode` answers the open ❓ in `06-password-login.md`: the client sends it in `RequestGameToken`, not in Connect.

## Reply parsing

Each parser creates a response object with the reply id and reads child elements of the reply root (`vt+72`).
A missing required element gives library result 6.

| Reply id | For | Elements read | Parser |
| ---: | --- | --- | --- |
| 3101 | LoginStart | `KeyData` (required, Base64) → SRP step 1 | `0x180003190` |
| 3102 | KeyData | `KeyData` (required). After it, SRP must be complete and have no pending output, else result 8. Then session key → cipher setup. | `0x180003270` |
| 3103 | LoginFinish | `UserId` (GUID text), `UserCenter`, `Roles/RoleId*` (int list), `AuthType` (int), `Status` (int), `PcCafe/PcCafeCode` (text). All optional. | `0x180003380` |
| 3107 | RequestGameToken | `Token` (required, GUID text) | `0x180003F30` |
| 3110 | RequestToken | `AuthnToken` (required, text) | `0x180004010` |
| 3114 | GetMyUserInfo | `UserId`, `UserCenter`, `UserName`, `LoginName` (required), `ExternalAccount` (optional, up to 4 nested values) | `0x180003900` |

GUID text: `sub_18002FF00` parses 16 hex bytes with `-` separators. The first three groups are byte-swapped into a Windows GUID (standard registry text form).

## Password proof exchange (SRP-6a family)

Code: `CSrpClient` (vtable `0x18005F388`), wrapped by `CNpSrpClient` (vtable `0x18005A968`).

| Item | Value | Source |
| --- | --- | --- |
| Hash H | SHA-256 (standard IV, big-endian digest bytes) | `0x18002E660`, `0x18002E4B0` |
| Group N | RFC 5054 1024-bit group | Table at `0x18005F2F0`, loaded by `0x18003F0E0` |
| Generator g | 2 | `0x18003F0E0` |
| k | `H(N ‖ g)` | `0x18003F0E0` |
| Inner hash | `H(I ‖ ":" ‖ P)`; I = login name, P = password | `0x18003ECC0` |
| x | `H(s ‖ H(I ":" P))` | `0x18003F4D0` |
| a | 32 random bytes | `0x18003F4D0` (RNG callback `a1+16`) |
| A | `g^a mod N` | `0x18003F4D0` |
| u | `H(A ‖ B)` | `0x18003F4D0` |
| S | `(B − k·g^x)^(a + u·x) mod N` | `0x18003F4D0` |
| K | 64 bytes: interleaved SHA-256 of the even and odd bytes of S (RFC 2945 `SHA_Interleave` form) | `0x18003EE10` |
| M1 | `H(H(N) xor H(g) ‖ H(I) ‖ s ‖ A ‖ B ‖ K)` | `0x18003F000`; `H(N) xor H(g)` at `0x180072E60` |
| M2 (server proof) | `H(A ‖ M1 ‖ K)` | `0x18003F3A0` |

### Number encoding (not standard RFC 5054)

The big-number library stores 32-bit little-endian words, least significant word first. Export (`0x1800316B0`) gives the raw word array: length = 4 × used words. Import (`0x180031A90`) reads little-endian words and trims zero high words.

- On the wire and in every hash input, A, B, s, N and g are **little-endian byte strings, padded to whole 32-bit words, high zero words removed**. g is hashed as 4 bytes (`02 00 00 00`).
- A hash output becomes a number by reversing its 32-bit word order and reading little-endian (`0x18003F4D0` reorders v21..v28 before import). The N table uses the same form.
- K input: zero bytes at the **start** of the little-endian S string are skipped while more than 4 bytes remain. The interleave then reads from the end of the string backwards. ❓ Confirm with a test vector against a real client.

❓ Check all of this section with one recorded real login before shipping. The wire order and the hash-to-number rule are read from code, not from a run.

### KeyData payloads

All `KeyData` text is Base64 (`sub_180030320` decode; `sub_1800375B0` encode). Inside, each field is `u32 little-endian length` + bytes.

| Direction | Message | Payload | Checks in client |
| --- | --- | --- | --- |
| Server → client | Reply 3101 to LoginStart | `[len][s] [len][B]` | Both lengths fit; payload fully used; B ≠ 0; B < N. Else step fails. |
| Client → server | Request `Auth/KeyData` | `[len][A] [32][M1]` | — |
| Server → client | Reply 3102 to KeyData | `[32][M2]` | Length is exactly 32; payload fully used; M2 matches. Else fail. |

SRP client states: 3 = idle, 0 = after start, 2 = waiting for M2, 3 = done (`0x18003FDB0`).

## Session encryption after login

After reply 3102 succeeds, `sub_180002190` creates two RC4 cipher objects (`sub_18002EC80`, 0x110-byte state), both keyed with the 64-byte K.
One is for sending (`a1+80`), one for receiving (`a1+72`). Each starts with a fresh state.
When the send cipher exists, `sub_18001C300` passes the whole built message through `sub_180018780`, so all later messages are encrypted.
❓ Whether the request line and headers are encrypted, or only the body, is not checked in `sub_180018780`.
❓ Whether the server must switch at the same message boundary (directly after its 3102 reply) is inferred, not tested.

## What a local STS server needs

| Step | Server action |
| --- | --- |
| LoginStart | Look up the verifier v and salt s for LoginName. Make b, B = k·v + g^b mod N. Reply 3101 with KeyData `[s][B]`. |
| KeyData | Decode `[A][M1]`. Reject A mod N = 0. Compute u, S = (A·v^u)^b mod N, K, expected M1. Reject on mismatch. Reply 3102 with `[M2]`. Start RC4 both ways with K. |
| LoginFinish | Reply 3103 with UserId, Roles, AuthType, Status. |
| RequestGameToken | Check GameCode and AccountAlias. Issue a single-use token GUID. Reply 3107 with `Token`. |

Verifier enrollment: v = g^x mod N with the x formula above. The server stores only s and v.

No key material from any vendor is needed. N and g are a public standard group. Every secret (s, v, b, K) belongs to the local server.
