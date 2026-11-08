# 04 — NP token validation by LobbyDaemon

## Sources and build separation

| Source | Build | Use |
| --- | --- | --- |
| `Docs/Server/LobbyDaemon/04-login.md` | 0.0.307 | Existing target login specification. Read first. Not re-derived here. |
| `ida/delegate/LobbyDaemon_348_copy.i64` | Guide labelled 348 | Named NPChannel / NPGameServiceChannel functions. Addresses below use this guide image. ❓ Exact file version was not checked. |
| `ida/delegate/sts/StsMsgSrv64.dll` plus PDB | ❓ Release identity unchecked | STS request builder, reply parser, API types |
| `Program/Server/LobbyDaemon/src/np.rs` | Local Rust code | Transport-free `Np` trait and deterministic `StubNp` |

Do not transfer a guide address to the build-307 image. Do not claim that the guide establishes all build-381 client behavior.

## Checked validation sequence

| Step | Checked behavior | Evidence |
| --- | --- | --- |
| 1. Client to LobbyGate | NP token and portalAccountId reach LobbyDaemon Init. Both GUIDs must be nonzero in the existing target specification. Init does not consume the token. | Build 307: `0x14003ACD0`, `0x140094D50`; existing `04-login.md` |
| 2. Start login | Start job calls consume with saved portal GUID, NP token, and client IPv4. Expiry event is 35, not a duration. | Guide: `StartCliSessionWnpTokenJob::consumeToken`, `0x1400D31E0` |
| 3. Build request | `portalAccountId` becomes `pUserId`; token becomes `pGameToken`. IPv4 becomes text at runtime. | Guide: `NPGameServiceChannel::SendRequestConsumeGameToken`, `0x1400D0220` |
| 4. Wire request | `POST /Auth/ConsumeGameToken STS/1.0`. Header `t` routes by portal GUID. XML `Request` has `Token`, `GameCode`, `ClientNetAddress`, in that order. | STS: `0x180001400`, command map `0x180028020` |
| 5. Correlate | NPChannel registers request command and context, then builds it. Reply handler requires response type, a transaction, and transaction kind 1. It dispatches event 34 to the saved job. | Guide: `NPChannel::SendMsg`, `0x140003AC0`; `Handle_SrvAuth_ConsumeGameToken`, `0x1400CBF40` |
| 6. Parse identity | Read `GameAccountId`, `LoginName`, `UserId`, `UserName`, `UserCenter`, and repeated `Roles/RoleId`. | STS: `ParseResponseAuthConsumeGameToken`, `0x18000DD10` |
| 7. Accept or reject | Require INIT.verify state `0x10002`. Accept status 200, or status 400 plus case-insensitive `Success`. Require non-null login-name and user-name pointers. Copy game-account GUID and user GUID from reply. Convert names from UTF-8. Copy roles. | Guide: `consumeTokenResponse`, `0x1400D3500`; build-307 equivalent already specified at `0x1400CECD0` |
| 8. Continue | Activity feature selects state 3; otherwise state 7 (presence login). Other platform steps can still block login. | Guide: `0x1400D3500`; target flow in existing `04-login.md` |

Status rejection gives LobbyGate Start-fail reason 15. Missing required name fields gives reason 2. Wrong session state gives reason 5. Guide: `0x1400D3500`. These are LobbyDaemon wire reasons, not STS status values.

## Which values must match

| Value | Observed validation request | Local rule / limit |
| --- | --- | --- |
| NP token | GUID in `Token` | ❓ Bind one issued token to one account and consume it once, atomically. Server-side NP consumption was not recovered. |
| portalAccountId | Routed `t` name from `pUserId` | ❓ Require equality with the stored issue record. The checked LobbyDaemon reply handler copies returned UserId; it does not compare it to the input portal GUID. The local service must enforce this binding. |
| Game code | `GameCode` from STS library initialization | ❓ Require equality with token issue policy. The original NP service check is not present in this DLL. |
| Client IPv4 | `ClientNetAddress` | ❓ Record for audit if configured. Do not require address equality by default without evidence. Never copy a sample address. |
| Server app ID | No AppId field in the checked ConsumeGameToken builder | ❓ Validate the trusted local caller against local policy. Do not invent an AppId XML field here. |
| Client app ID | RequestToken API has appId; TokenKeyData API has AppId | ❓ Bind issue records to configured app policy. Exact issue wire field and game-token binding remain open. |
| Program ID | Appears in game-server Connect as `Program` | ❓ Check an allowed local role at connection setup. It is not an account identifier. |
| Returned GameAccountId | Copied into job identity | Preserve a stable generated game-account GUID. Do not equate it with portal GUID. |

No original token lifetime or server-side replay rule was established. Single use and short local lifetimes are proposed local security policy.

## Integration with the Rust stub

The local trait is [`Np`](../../../Program/Server/LobbyDaemon/src/np.rs):

```rust
fn consume(&self, token: [u8; 16], portal: [u8; 16]) -> NpFuture<'_, Identity>;
```

`Identity` has account GUID, UTF-16 login/user names, slot ID, profile, and grades. The stub has generated identity fixtures keyed by `(token, portal)`. It rejects unknown tokens and repeat consumption. Optional generated deadlines use Tokio time. It rejects zero account GUID or empty names. Those are **local rules**, not all recovered NP rules.

❓ Proposed integration: implement a new `Np` adapter over a shared local token authority. Keep `StubNp` unchanged for existing tests. The adapter must validate token, portal GUID, expiry, and game/app policy in one operation. It must return a complete `Identity`, or an explicit `NpError`. It must not call the vendor platform.

The trait has no per-call game code or app IDs. ❓ Bind these values to adapter construction. Do not change the trait silently. Presence login/logout and later profile/grade/slot behavior also need explicit local policy before the adapter can replace all stub operations.

❓ A native StsMsgSrv-compatible validation endpoint is a later adapter. A private local Rust interface is not the original NP ABI. No native endpoint is implemented in this step.
