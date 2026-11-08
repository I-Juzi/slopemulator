# 07 — Client login wire format

## Source

Source C64: `StsMsgCli64.dll` from the 2020 UE4 client (BNSR), file date 2020-01-14, 488 960 bytes.
Local copy: `ida/sts-client/cli64/StsMsgCli64.dll`. Image base `0x180000000`. No PDB (own PDB path: `StsMsgCli64.pdb`, GUID e977af1a847f44579f9cbef2c054c4af, not available).
Source file names in the DLL: `StsMsgCli.cpp`, `Services/Srp/Srp.cpp`, `Services/SrpWrapper/SrpWrapper.cpp`, `Services/Crypt/CptRc4.cpp`, `Services/Base64/Base64.cpp`.
`StsMsgCli32.dll` (UE3 client, operator copy `~/Downloads/stuff/`, 395 776 bytes, version 0.122, PE timestamp `0x5EB3B1C6`): token verify key is the same RSA-2048 key as in `StsMsgCli64.dll`, at file offset `0x4A088` (360 Base64 chars, type 4, e 65537). ❓ The wire code is not compared yet.
The operator's UE3 `StsMsgCli64.dll` (PE timestamp `0x5EB3B23C`) is a later build than the local copy (`0x5E140714`). Same version 0.122, same key at `0x58D60`. ❓ Code differences are not checked.

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
- K input (verified, `StsMsgCli32.dll` 0.122 `sub_10031040`, loop at `0x10031061`): the client skips bytes at the **start** of the little-endian S string while they are **not** zero and more than 4 bytes remain. Hashing starts at the first zero byte. If the remaining length is odd, one more byte is removed. The interleave then reads from the end of the string backwards. This is not RFC 2945. A server that strips leading zeros instead computes a different K, and the client rejects nothing visibly: the server rejects M1. Checked by emulating the DLL against `sts-local` (3 of 3 logins, server accepts M1, DLL accepts M2), 2026-11-08.

❓ Check all of this section with one recorded real login before shipping. The wire order and the hash-to-number rule are read from code, not from a run.

### KeyData payloads

All `KeyData` text is Base64 (`sub_180030320` decode; `sub_1800375B0` encode). Inside, each field is `u32 little-endian length` + bytes.

| Direction | Message | Payload | Checks in client |
| --- | --- | --- | --- |
| Server → client | Reply 3101 to LoginStart | `[len][s] [len][B]` | Both lengths fit; payload fully used; B ≠ 0; B < N. Else step fails. |
| Client → server | Request `Auth/KeyData` | `[len][A] [32][M1]` | — |
| Server → client | Reply 3102 to KeyData | `[32][M2]` | Length is exactly 32; payload fully used; M2 matches. Else fail. |

SRP client states: 3 = idle, 0 = after start, 2 = waiting for M2, 3 = done (`0x18003FDB0`).

### 32-bit client (`StsMsgCli32.dll` 0.122, build 381)

Source: `ida/sts-client/cli32/StsMsgCli32.dll` (SHA-256 `04b7e557…5433`, the same file as the operator's `BNS_Client\bin`). Image base `0x10000000`.

| Item | 32-bit result | Source |
| --- | --- | --- |
| SRP group N | Same RFC 5054 1024-bit N, stored as 32 reversed words | Table `0x1004F7DC`..`0x1004F85C`, loader `sub_10031280` |
| Name before hash | ASCII upper case → lower case, 64 bytes max | `sub_10023610` → `sub_100253C0` |
| Inner hash | `H(name ‖ ":" ‖ password)`, 32 bytes | `sub_10030F20` |
| Step on reply 3101 | Base64 decode, then `[len][s][len][B]`, exact length, B ≠ 0, B < N | Parser `sub_10003B90` → `sub_100023F0` → `sub_10031A80` → `sub_10031590` |
| Step result on 3101 | **Ignored.** The parser returns 1 even if the SRP step fails. Then the KeyData request builder `sub_10016890` finds no pending output and returns 8 without a send. | `sub_10003B90`, `sub_10016890` |
| Step on reply 3102 | M2 check; failure returns 8 | `sub_10003C40`, `sub_10031410` |
| Ping | One-way; no reply parser | `sub_100231E0` |

Consequence: a bad reply 3101 does not give a visible error. The client goes idle and later sends `Sts/Ping`.

Live login 2026-11-08 19:07 (wire capture through the forwarder): Connect → `<Reply/>`; LoginStart → KeyData (salt 32 bytes, B 128 bytes, B < N in both byte orders); then only `Sts/Ping` after 8 s. The old local server closed the socket on that Ping. ❓ Not yet known if the client was still about to send KeyData, or if its SRP step failed.

Game side (`Client.exe` 2020 build with PDB, ❓ not the laptop build): the response dispatcher `sub_C25140` handles a reply only with status `200` and reason `OK`. Reply 3101 goes to `sub_C16AF0` → `sub_C074B0`. That function builds an `NPRequestPacket` (command `0xC1E`) and sends it through `[NPSession+0x170]` vt+4. It sends only when the byte `[NPSession+0x174]` is set; `NPSession::OnConnect` sets it. ❓ Not confirmed on the laptop build why no KeyData was sent at 19:27.

## Session encryption after login

After reply 3102 succeeds, `sub_180002190` creates two RC4 cipher objects (`sub_18002EC80`, 0x110-byte state), both keyed with the 64-byte K.
One is for sending (`a1+80`), one for receiving (`a1+72`). Each starts with a fresh state.
RC4 covers the **whole byte stream**: request line, headers and body.

| Direction | Function | What it does |
| --- | --- | --- |
| Send | `sub_18001C300` → `sub_180018780` | Takes every segment of the serialized message (request line, headers, body) in order and runs them through one continuous RC4 stream (`a1+80`, vt+8). |
| Receive | `sub_1800196A0` | Runs each raw received chunk through the receive RC4 (`a1+72`) **before** it is appended to the frame buffer (`a1+40`, `sub_18002B7B0`). The frame parser sees only plaintext. |

The RC4 state is never reset per message. It is one stream per direction for the rest of the connection.
Switch point: the client creates the ciphers while it handles reply 3102. So reply 3102 itself arrives in plaintext, and the first encrypted bytes are the client's next request (`LoginFinish`). The server must switch both directions directly after it sends 3102.
❓ The switch point is read from code; confirm it with a real client.

## Token login path (not used by the local server)

Enum 47 `Auth/LoginTokenStart` and enum 48 `Auth/TokenKeyData` form a separate login that uses a token from the launcher instead of a password. It is a TLS 1.0-style handshake:

| Step | Message | Content | Source |
| --- | --- | --- | --- |
| 1 | Client → `LoginTokenStart` | `<ClientRand>` (random bytes) | `0x18001D530` |
| 2 | Server → reply 3111 | `<ServerRand>`, `<ServerPublicKey>`, `<ServerSignature>` (all required) | `0x180004100` |
| 3 | Client check | Verifies `ServerSignature` over `ServerPublicKey` with a verification key built into the client (`sub_180001900`). Failure → result 8, login stops. | `0x180004100` |
| 4 | Client → `TokenKeyData` | `<PremasterSecret>` and `<AuthnToken>`, both encrypted with `ServerPublicKey`; `<AuthProviderCode>`, `<AppId>` | `0x18001D670` |
| 5 | Both | Key block from a TLS-style PRF over premaster, ClientRand and ServerRand (`sub_1800013E0`, suite id 4). Separate RC4 keys for send and receive. | `0x1800135A0` |

A local server cannot pass step 3 without the vendor's signing key, or without a change to the client. This path is out of scope. The local server uses the password path (SRP) only, which needs no vendor key.

## What a local STS server needs

| Step | Server action |
| --- | --- |
| LoginStart | Look up the verifier v and salt s for LoginName. Make b, B = k·v + g^b mod N. Reply 3101 with KeyData `[s][B]`. |
| KeyData | Decode `[A][M1]`. Reject A mod N = 0. Compute u, S = (A·v^u)^b mod N, K, expected M1. Reject on mismatch. Reply 3102 with `[M2]`. Start RC4 both ways with K. |
| LoginFinish | Reply 3103 with UserId, Roles, AuthType, Status. |
| RequestGameToken | Check GameCode and AccountAlias. Issue a single-use token GUID. Reply 3107 with `Token`. |

Verifier enrollment: v = g^x mod N with the x formula above. The server stores only s and v.

No key material from any vendor is needed. N and g are a public standard group. Every secret (s, v, b, K) belongs to the local server.

## st3 — Local Rust implementation

Source: `Program/Platform/Sts`, implemented from this document. Build target: source C64 above (2020-01-14). No binary was opened in st3.

| Part | Local implementation | Scope |
| --- | --- | --- |
| SRP | `src/srp6.rs`; `password::SrpPasswordBackend` | SHA-256, public RFC 5054 group, word encoding, enrollment, both server steps. The server stores no password. |
| KeyData | `src/key_data.rs` | Base64 and LE length fields. Reject truncated fields and trailing bytes. Check B range and exact 32-byte proofs. |
| XML | `src/messages/client.rs` | All seven request bodies and the documented reply fields. GUID bytes use Windows memory order. Text uses registry order. |
| Stream cipher | `src/rc4.rs`; `src/wire.rs` | Two continuous RC4 states. Each uses K directly. No initial bytes are discarded. Reply 3102 stays plaintext. |
| Game token | `src/tokens.rs`; `src/wire.rs` | Check GameCode and AccountAlias before issue. Keep the existing single-use authority and binding checks. |
| App token | `wire::AppTokenIssuer` | An explicit backend must check the AppId grant. Without that backend, RequestToken fails closed. Do not return a game GUID as AuthnToken. |
| Listener | `src/listener.rs`; `src/bin/sts-local.rs` | Blocking std::net binary behind feature `listener`. Accept only an explicit loopback bind address. |

### Local policies and open checks

- ❓ Check SRP number encoding, hash-to-number word reversal and K against one real login. The independent test client checks the transcription, not vendor interoperability.
- The interleave rule is verified against the 32-bit client DLL. See the K input rule above (non-zero skip, not zero removal).
- ❓ `CipherScope::WholeMessage` and `CipherScope::BodyOnly` form one switch for both directions. Select it explicitly. Confirm the scope and the exact switch boundary with the client.
- ❓ The local XML serializer uses `<Reply>` and success status `200 OK`. The source above does not establish the reply-root name or the success status.
- XML layout (verified, `StsMsgCli32.dll` 0.122): write one element per line, as the DLL's own writer does (`<Reply>\n<KeyData>..</KeyData>\n</Reply>\n`). The DLL does not find `<KeyData>` directly after `<Reply>` without a newline: parser `sub_10003B90` returns 6, the SRP step does not run, and the game sends no KeyData. It then only sends `Sts/Ping`. Found by emulating the DLL on the captured reply of the 2026-11-08 19:46 login; the same reply with newlines parses and the DLL builds `Auth/KeyData`.
- ❓ Logout uses an empty `<Reply/>`. Its reply ID and original reply fields remain unknown.
- ❓ UserCenter, roles, AuthType, Status and PcCafe values are local policy. The session returns configured UserCenter, no roles, AuthType 0 and Status 0. Confirm which values the client needs.
- ❓ UTF-8 is the local XML encoding. Confirm non-ASCII AccountAlias and AppId text with the client.
- Local XML policy rejects DTDs, unknown entities, attributes, duplicate scalar fields, unexpected elements, extra roots and nesting deeper than eight elements. Local limits are 64 KiB per XML body and 128 elements. These are not claims about the original parser.
- Local KeyData limits allow a salt of up to 1024 bytes and a public number of up to 128 bytes. Number fields contain whole u32 words. The enrolled salt and verifier use canonical word form.
- BigUint arithmetic is variable-time. This implementation has no independent audit. Use it for the local test service only.

### Listener configuration

The listener has no address or account defaults. Every flag below is required.

| Flag | Value |
| --- | --- |
| `--bind` | An explicit loopback SocketAddr. No remote bind is accepted. |
| `--program` | A positive local program ID. |
| `--game-code` | The granted game code, 1..8 ASCII letters or digits. |
| `--user-center` | The local UserCenter value. |
| `--accounts` | A local verifier TSV file. See the columns below. |
| `--token-seconds` | A positive game-token lifetime. |
| `--token-capacity` | A positive token entry limit. |
| `--session-seconds` | A positive absolute connection lifetime, including partial input. |
| `--min-accept-ms` | A positive global interval between accepted login attempts. |
| `--cipher-scope` | `whole` or `body`. ❓ Select after a client check. |
| `--connect-mode` | `login-start` or `unverified-connect`. See below. |

❓ The client Connect body remains unspecified. `login-start` expects another caller to handle Connect. `unverified-connect` accepts the command `Sts/Connect` without body checks. It returns an empty local reply. This is an explicit test mode, not a recovered Connect implementation. The service still uses its configured program/game binding. The game-token request must match the configured game code and the account grant.

The TSV has nine columns: LoginName, UserName, portal GUID, account GUID, slot ID, AccountAlias, salt Base64, verifier Base64, scheme. Use `password::VerifierRecord::enroll` to make local records. The scheme is `sts-2020-sha256-word-srp6a`. Do not add a password column. Do not commit the file.

The listener serves one connection at a time. It sets read and write timeouts from an absolute deadline. It rejects new connections inside the configured global accept interval. Session errors, disconnect and drop revoke unused game tokens. The stand-alone listener has no AppTokenIssuer and no game-server token-consume transport. An embedding caller can share TokenAuthority with the existing NP adapter. The default workspace checks compile the listener module in the library test target. The binary still requires feature `listener`.

## After LoginFinish (game side, 2026-11-08)

`NPSession::ReceiveLoginFinishSuccess` (`Client.exe` 2020 PDB build, 0xC24D80) sends three requests in this order on the normal path. ❓ Order not yet confirmed on the laptop build wire.

| Order | Game sender | Game ECommand | Wire request | Reply parser (`StsMsgCli32.dll` 0.122) | Reply body |
| --- | --- | --- | --- | --- | --- |
| 1 | `sub_C06D60` | 0xC22 = 3106 | `POST /GameAccount/ListMyAccounts`, body `GameCode` (builder `sub_10023F10`) | `sub_10004360` | Repeated `GameAccount`, each with `Alias`. Only `Alias` is read. |
| 2 | `sub_C072A0` | 0xC2A = 3114 | `POST /Auth/GetMyUserInfo`, empty `Request` (builder `sub_10016F80`) | `sub_10004130` | See 3114 above. |
| 3 | `NPSession::SendTokenRequest` (0xC24710) | 0xC26 = 3110 | `POST /Auth/RequestToken`, body `AppId` (builder `sub_100172A0`) | `sub_10004660` | `AuthnToken` (required). Missing → result 6. |

The live login of 19:56 closed with `Login(Unavailable)` after LoginFinish. `sts-local` then had no ListMyAccounts handler and no AppId token issuer.

Local server policy:

- `GameAccount/ListMyAccounts`: check `GameCode` against the binding. Reply with the aliases of the account grants for the binding.
- `Auth/RequestToken`: `sts-local --app-ids <list|*>` issues a random 64-hex opaque token and logs the AppId. ❓ The AppId values of build 381 and the vendor token format are not known. Nothing consumes the token yet.

### VirtualCurrency/GetBalance (2026-11-08)

| Item | Value | Source |
| --- | --- | --- |
| Request | `VirtualCurrency/GetBalance`, sent after LoginFinish | live login 20:08:50, sts-local log |
| Library IDs | 3957, 3970, 3981 | StsMsgCli32.dll 0.122 builders `sub_10019A30`, `sub_1001A690`, `sub_1001B740` |
| Request body | `UserId`, `CurrencyGroupId`/`CurrencyGroupUnit` or `CurrencyId`, optional `ClientNetAddress` | same builders |
| Reply body | `BalanceInfoListCount`; if > 0, `BalanceInfoList/BalanceInfo{CurrencyGroupId, CurrencyId, Balance, Expired}` with exactly that many entries | parser `sub_10008270` (3957); `sub_10009960`, `sub_10009EE0` read the same names |
| Local policy | Reply `BalanceInfoListCount` = 0. The local server keeps no wallet. | `wire.rs` |

❓ Which of the three IDs the game uses is not known. The reply is matched by transaction, so the local reply does not depend on it.

### Slot/ListSlots and unknown requests (2026-11-08)

| Item | Value | Source |
| --- | --- | --- |
| `Slot/ListSlots` request | `UserId`, `SlotType`; library ID 1501 | StsMsgCli32.dll 0.122 builder `sub_10017B60`; live login 20:14:15 |
| `ListSlots` reply | repeated `Slot{SlotId, SlotType, SystemSlot, SlotData, Changed, Registered}`; zero `Slot` is valid | parser `sub_10005B40` |
| Local `ListSlots` policy | reply `<Reply/>` (no slots) | `wire.rs` |
| Error replies | the client reads `Error/@code` when status >= 400, and still passes the reply to the game | `sub_100039A0` |
| Status table | `200`; `400 ErrBadData` code 4 | table at 0x100582D4 (`sub_10017F40`) |
| Local policy for unknown requests | after login: `STS/1.0 400 ErrBadData`, body `<Error code="4"/>`, session stays open, log `sts-local: X not supported`. Before login: close the session. | `wire.rs`, `listener.rs` |

❓ How the game reacts to a 400 reply for each request is not verified.
