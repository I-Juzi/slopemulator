# 06 — Password login

## Sources and limits

Source S: `ida/delegate/sts/StsMsgSrv64.dll` and its adjacent PDB.
Image base: `0x180000000`. ❓ DLL release identity and build-381 match remain unknown.
Source C: the operator-supplied client path in `03-login-flow.md`.
No client binary was opened in this job.

The DLL is a game-server message library. It is not the platform authentication service.
`StsMsgSrv::Build` at `0x180016D70` has no cases for client password commands 3101..3103 or token issue 3107/3110.
`Parse` at `0x18001ACF0` has no client password response handlers.
Do not treat the shared PDB records as recovered XML schemas.

## Password path map

| Step | Checked evidence | Missing evidence |
| --- | --- | --- |
| Connect | `0x1800010F0`: server-library `Sts/Connect` writes ConnType, Address, ProductType, AppIndex, Epoch, Program, Build, Process. | ❓ Client ConnType, fields, reply and server state checks. Do not use server ConnType 401 for a client. |
| LoginStart | `Cli2Auth::CommandToString`, `0x180028020`: `Auth/LoginStart`, enum 25. PDB request has login-name and password pointers. Response has only its base. | ❓ Wire fields and whether any password bytes are sent. API password input does not prove plaintext transmission. |
| KeyData | `0x180028020`: `Auth/KeyData`, enum 22. Login-key-data request and response have only their bases. `CNpSrpClient::ServerKeyData`, `0x18002B730`, forwards an opaque buffer and completion flag. | ❓ XML/binary envelope, round count, placement of public values and proofs. Empty API records do not prove empty bodies. |
| LoginFinish | `0x180028020`: `Auth/LoginFinish`, enum 24. Request has language enum. Reply has user GUID, role count/IDs, auth type, status and cafe code. | ❓ Wire fields, proof placement, status codes and service state guards. |
| RequestGameToken | `0x180028020`: enum 35. API request has wide game-alias text. Reply has game-token GUID. | ❓ Wire schema and build-381 call selection. |
| RequestToken | `0x180028020`: enum 46. API request has wide app-ID text. Reply has wide authentication-token text. | ❓ Wire schema and build-381 call selection. Do not equate this token with the game-token GUID. |
| Logout | `0x180028020`: `Auth/LogoutMyClient`, enum 28. PDB `_StsMsgRequestLogOut` has only its request base. | ❓ Wire body, reply, revocation and allowed states. Channel `Logout` is a different operation. |

PDB `ELanguageType` names English through Russian as 0..10.
It also names values 16 and 17. ❓ Their client use is unknown.
The integers above are ABI enums, not numeric wire commands.

Source C establishes Connect -> LoginStart -> SendLoginRequest -> token request -> token success.
❓ KeyData and LoginFinish placement follow shared API names, not recovered platform control flow.
No signature-bearing token-login operation is part of this job.

## Authentication boundary

| Evidence | Algorithm family and step | Secret ownership |
| --- | --- | --- |
| `Initialize`, `0x180019020` | Creates `NewNpSrpClient` at library initialization. | Client-side password input exists in the API. |
| `NewNpSrpClient`, `0x18002B8C0` | Wraps `NewSrpClient`; source path names `Services/SrpWrapper/SrpWrapper.cpp`. | No key material was read. |
| PDB `ISrpServer_vtbl` | StartKeyExchange, ClientKeyData, GetSessionKey. | ❓ Server verifier and salt roles follow SRP conventions; parameter semantics are not established by unnamed pointer types. |
| `CSrpClient::ChalM2` `0x18003C930`, `ChalSaltB` `0x18003CAF0`, `GenVerifier` `0x18003D140` | Symbol names support SRP-family proofs and verifier data. No implementation was read. | ❓ Vendor proof encoding and password preprocessing remain unknown. |

❓ The approved evidence does not identify SRP-6a, RFC 5054, a group size or the login hash.
A SHA-256 function elsewhere in the DLL does not establish the login hash.
Do not reverse-engineer vendor cryptographic internals to fill these gaps.
No vendor-secret dependency was established. No licence or anti-cheat boundary was encountered.

## Program and game binding

| Field | Checked carrier | Required local check |
| --- | --- | --- |
| Program | Server-library Connect XML `Program`, `0x1800010F0`, from Initialize. | Compare with an explicit allowed program ID. |
| GameCode | `Auth/ConsumeGameToken` XML `GameCode`, `0x180001400`, from stored initialization data. | Compare with the issued token's game binding. |
| Game alias | PDB RequestGameToken API input. | Compare with an account grant. |
| App ID | PDB RequestToken API input. | Compare with an account grant. |

`Initialize`, `0x180019020`, accepts nonnegative program IDs and game-code strings of 1..8 bytes.
Source C says the client password path requires nonzero NP program ID and game code; otherwise error 105.
❓ The client wire carrier of GameCode is not established. Do not infer a client Connect GameCode field from the consume-token builder.
❓ Original server allowlists, error codes and cross-field checks remain unknown.
Library result 3 means wrong game code, 4 wrong parameters, 8 wrong key data and 9 wrong command.
These are not platform error codes. See `02-messages.md`.

## Local test profile — not the original wire protocol

The `srp` 0.6.0 README, lines 24..31, states that interoperability is untested and no independent audit exists.
The trial implementation was removed. The final crate does not depend on `srp` or `sha2`.
The requested audited-backend condition blocks real password authentication.
❓ SRP-6a with an RFC 5054 group remains a possible future profile, not a verified vendor standard.
No group size or hash is selected by the final code.

The crate supplies a trusted `PasswordBackend` / `PasswordExchange` interface.
`Session::new` uses `UnavailablePasswordBackend` and rejects login with Unavailable.
An audited backend must validate the scheme, verifier, public values and proofs.
`Account::from_verifier` imports a named salt/verifier record. It does not enroll passwords.
The in-memory account store has no password field.
Only the test binary supplies `GeneratedBackend`. It selects explicit generated success/failure fixtures.
These tests do not verify real password authentication or published-standard interoperability.

The caller supplies a cryptographic RNG and monotonic time.
Use an OS-backed RNG outside tests. Test RNG seeds and all identity data are generated.
Keep authentication-token strings and game-token IDs as distinct types.

| Local state | Request | Guard | Next state |
| --- | --- | --- | --- |
| New | Connect | Exact configured program/game binding | Connected |
| Connected | LoginStart | Known account; allowed binding | AwaitKeyData |
| AwaitKeyData | KeyData | Valid public data and client proof | AwaitFinish |
| AwaitFinish | LoginFinish | Valid language; proof already checked | Authenticated |
| Authenticated | RequestGameToken | Exact granted game alias | Authenticated |
| Any open state | Logout | No extra guard | Closed |
| Any | Wrong-order request or failed authentication | Fail closed | Closed |

Local LoginStart returns salt and server public data.
KeyData takes the client public value and proof. It returns the server proof.
❓ This message split is a local API choice, not a recovered KeyData wire exchange.
Unknown users and bad proofs return one local authentication error. Timing equality is not claimed.
Issue tokens only after LoginFinish and an explicit game-token request.
Bind each token to portal ID, account ID, program ID and game code.
Reject at the exact expiry deadline. Consume a valid token once under one lock.
Wrong binding must not consume a token.
Logout, protocol failure and Session drop revoke unconsumed tokens from that session.
The authority has an explicit positive lifetime and entry limit.
The adapter lives in `crates/sts/src/lobbyd_adapter.rs`, behind feature `lobbyd-adapter`.
It implements `lobbyd::np::Np`. It consumes tokens only from this authority.
It maps generated identity fields to LobbyDaemon Identity.
It delegates presence, name and world operations to an explicit backend.
It never falls back to that backend for a rejected token.
No LobbyDaemon source or default behavior changes.

No local authentication-token-string issuer is specified until its consumer is known.
No XML login serializer or TCP listener is specified while the original body schema is unknown.
❓ Original account grants, token lifetime, GUID wire order and session timeouts remain unknown.
The local pure session has no attempt deadline or rate limiter. Add both before any listener.
Do not deploy the test backend or treat fixture proofs as password authentication.
