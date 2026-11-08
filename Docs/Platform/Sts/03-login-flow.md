# 03 — Login flow and session state

## Evidence levels

Source S: STS DLL and PDB, as defined in `01-transport.md`.
Source C: the client symbol sequence supplied in the task. No client executable or client STS DLL was read.

The server DLL exposes shared client message types. It does not supply the checked client builders or the authentication service handlers. Thus this file gives a **partial protocol model**, not a tested login recipe for build 381.

## Operations and data

| Operation | Checked source | What is known | What is not known |
| --- | --- | --- | --- |
| Initial connect | S `0x1800010F0` | Game-server Connect body; see `02-messages.md` | ❓ Client ConnType, body, and reply |
| Password start | S PDB `_StsMsgRequestLoginStart`; command table `0x180028020` | API accepts login name and password. Library command 3101 names LOGIN_START. | ❓ Wire fields, password encoding, challenge order |
| Password exchange | S PDB LOGIN_KEY_DATA 3102; LOGIN_FINISH 3103 | Key-data request and response types exist. Finish request has a language enum. Finish response has user GUID, role data, auth type, status, and cafe code. | ❓ Exact exchange and wire fields |
| Token start | S PDB LOGIN_TOKEN_START 3111; TOKEN_KEY_DATA 3112 | Start response has `serverRand`, `serverPublicKey`, and `serverSignature` fields. Token-key-data request has an authentication-token byte buffer, Base64 flag, provider code, and AppId. | ❓ Binding to launcher `/SessKey`, verification rules, wire fields |
| Token request | S PDB REQUEST_GAME_TOKEN 3107; REQUEST_TOKEN 3110 | First API takes game alias and returns a game-token GUID. Second takes appId and returns an authentication-token string. | ❓ Which request build 381 uses at each stage |
| Client success | C `ReceiveTokenSuccess` | Client symbol names a token-success step | ❓ Exact reply schema and state transition |
| Lobby login | Existing build-307 spec `04-login.md` | Init stores NP token and portal GUID. Start consumes the token. | ❓ End-to-end build-381 acceptance remains untested |

These PDB enum integers belong to the library ABI. They are not numeric fields in a wire frame.

## Partial state model

Every transition with ❓ is a proposed model from symbol names or API types. It is not recovered control flow.

| State | Input / operation | Guard | Next state |
| --- | --- | --- | --- |
| ❓ New | Client Connect | ❓ Supported connection metadata | ❓ Connected |
| ❓ Connected | LoginStart | ❓ Local password path selected | ❓ Password exchange |
| ❓ Password exchange | KeyData, then LoginFinish | ❓ Successful exchange | ❓ Authenticated |
| ❓ Connected | LoginTokenStart | ❓ Launcher token path selected | ❓ Token exchange |
| ❓ Token exchange | TokenKeyData | ❓ Valid, unexpired single-use launcher session key; trusted server exchange | ❓ Authenticated |
| ❓ Authenticated | RequestGameToken | ❓ Authorized game binding | ❓ Game token issued |
| ❓ Authenticated | RequestToken | ❓ Authorized app binding | ❓ Authentication token issued |
| ❓ Game token issued | Client sends NP token and portalAccountId to LobbyGate | Existing LobbyDaemon Init/Start rules apply | LobbyDaemon consumes token |
| ❓ Any unfinished state | Failure, timeout, disconnect, or LoginCancel | ❓ Reject pending work | ❓ Closed |

Do not collapse authentication tokens, launcher session keys, NP game-token GUIDs, lobby tokens, or world-entry tokens into one type. The existing LobbyDaemon lobby-token desk is a different service.

## Cryptographic boundary

| Evidence | Algorithm family / step | Secret ownership / limit |
| --- | --- | --- |
| S symbols `CNpSrpClient`, `CSrpClient`; `Initialize` `0x180019020` creates an NP SRP client | SRP family. ❓ Exact placement in the build-381 password path is not established. | ❓ Normal SRP roles imply client password knowledge and server verifier storage. This binary pass did not verify server storage. |
| S `CNpSrpClient::ServerKeyData` `0x18002B730`, `GetSessionKey` `0x18002B7B0` | Symbol-level key exchange and session-key boundary only | ❓ Runtime key ownership and use in transport remain unchecked. No implementation was read. |
| S PDB `_StsMsgResponseLoginTokenStart` | Signature-bearing server challenge at token start. ❓ Signature algorithm is not named by this type. | ❓ Server must hold the signing secret for a verified signature scheme. Client trust source and vendor-secret dependency are not established. |
| S `CryptDigestSha256` symbol `0x18003FF00` | SHA-256 family exists in this DLL | ❓ This does not prove its use in login or token signing. |

No key material or cryptographic internals appear in this spec. Stop the token-login path at the signature boundary. Do not reproduce a vendor signature or disable client verification. If later approved evidence proves that the client requires an unavailable vendor secret, record a hard compatibility boundary and stop that path.

No licence-check or anti-cheat function was inspected in this pass. No address for such a boundary was encountered in the selected functions. Do not open BloodCert or BNSHelper.dll.

## Proposed per-session state

❓ Local design, not recovered server layout:

| Record | Required local data |
| --- | --- |
| Connection | Protocol state, bounded input buffer, pending transaction IDs, sequence/completion state, idle deadline |
| Login attempt | Selected path, requested app/game binding, attempt deadline, resolved account reference |
| Launcher key | Stored key digest, account reference, app binding, issued time, expiry, consumed flag |
| Authenticated session | Portal GUID, game-account GUID, roles, local session ID, revocation state |
| Token issue | Opaque game-token GUID, portal/account binding, game code, app identifiers, expiry, consumed flag |

Never log passwords, launcher keys, authentication tokens, game tokens, or challenge buffers. The first Rust crate stores opaque bodies only. It does not claim to implement this state machine.

## Client-side path choice (build 381, read by the operator's assistant, 2026-11-08)

Source: static read of `ida/client2020/Client.exe` with its PDB names. No client code was changed.

| Step | Client function | Address | Fact |
| --- | --- | --- | --- |
| Address source | `Universe::SetUniverseInfo` | 0x440150 | NP and LobbyGate address/port come from `release.config2.xml` (`GReleaseConfig`), not from the command line. |
| Branch | `NPSession::OnConnect` | 0xC22150 | If the launcher/web flag (byte at `EInterfaceSetClientInfo`) is set: `SendInitialConnectMessage` then `SendWebLoginTokenStart` (the signature path). Else: `SendInitialConnectMessage` then `SendLoginStartMessage`. |
| Password path guard | `NPSession::OnConnect` | 0xC22150 | The password path needs a non-zero NP program id and NP game code (release config). Error 105 otherwise. |
| Password entry | `TitleLoginPanel::loginReq` → `Lobby::RequestLogin` → `sub_BDD570` → `NPSession::SendLoginRequest` | 0x7FF420, 0xCE5910, 0xBDD570, 0xBD40C0 | User name and password typed in the client login panel. |
| Replies the client handles | `ReceiveLoginFinishSuccess`, `ReceiveLoginFailure`, `ReceiveTokenSuccess`, `ReceiveTokenFailure` | — | Names only. |

Consequence: the password path does not use `LoginTokenStart` and its `serverSignature`. ❓ The password path still has a key-data step between LoginStart and LoginFinish (likely SRP-6a, a public standard where the server holds a verifier). This is not verified. ❓ Which command-line or config value sets the launcher/web flag is not verified.

### What sets the launcher/web flag (build 381)

| Source | Address | Fact |
| --- | --- | --- |
| `BNSClient::MainFramework` | 0x11907EF–0x11907FE | Flag = 1 only when `GRuleConfig->_useWebLaunching` (+0x37) is 1 **and** `/LoginMode` was not on the command line. |
| `BNSClient::DoCommandLine` | 0x113F1C4–0x113F1D5 | Any argument that starts with `/LoginMode` (10 chars compared) sets `byte_1A4C310 = 1`. The value after it is not read here. |
| `InitializeTCLS` | 0x114C570 | Also sets the flag (Tencent publisher path only). |
| `BNSClient::DoCommandLine` | 0x113F219, 0x113F2F8 | `/UserName:` → `BNSClient::SetWebLoginUserName`; `/Password:` is parsed next to it. These feed the web-login path, not the login panel ❓. |
| `Lobby::RequestLogin` | 0xCE5910 | If the typed user name has no separator (`SubStr`, likely `@` ❓), the client appends `GRuleConfig->_npAccountPostfix` (+0x3C). |

Consequence: start the client with `/LoginMode` (the BnsLauncher private-server sample uses `/LaunchByLauncher /LoginMode 2`). The client then uses the password path and shows its own login panel. Our STS must accept `user` or `user<postfix>`.
