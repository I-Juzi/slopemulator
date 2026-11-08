# 02 — Message inventory

Source S is the STS DLL defined in [01-transport.md](01-transport.md). ❓ Its exact release match to client build 381 is open.
The table separates library command IDs from wire commands. The wire carries names, not these enum integers.

## Checked request / reply bodies

| Operation | Request fields, in builder order | Reply fields read by parser | Source S |
| --- | --- | --- | --- |
| `Sts/Connect` (1001) | XML `Connect`: `ConnType` integer 401; `Address` text from caller; `ProductType` integer 0; `AppIndex` integer 1; `Epoch` seconds since 2001 UTC; `Program` integer from Initialize; `Build` integer 4000; `Process` process ID | ❓ No client Connect reply schema established | Builder `0x1800010F0` |
| `Sts/Ping` (1002) | Empty body; no transaction in this builder | ❓ Reply behavior unchecked | `0x1800012F0` |
| `Auth/ConsumeGameToken` (2101; Cli2Auth enum 12) | Header `t`: user GUID routed name. XML `Request`: `Token` GUID text; `GameCode` text; `ClientNetAddress` text | `GameAccountId` GUID; `LoginName` text; `UserId` GUID; `UserName` text; `UserCenter` text; `Roles/RoleId` repeated unsigned integers | Builder `0x180001400`; parser `0x18000DD10` |
| `Auth/LoginWebStart` (2105; Cli2Auth enum 26) | Optional routed portal GUID in `t`. XML `Request`: required `LoginName` text; optional `Password` text | Required `UserId` GUID; optional `UserName` text; `UserCenter` integer; `AuthType` integer | Builder `0x18000A100`; parser `0x18000E0B0` |
| Presence login (2201) | XML `Request`: `LoginName` text; `UserName` text; `GameAccountId` GUID; `Alias` text; `ClientNetAddress` text | Optional `UserId` GUID, `UserCenter` integer, `ExternalAccount/{AuthProviderCode, ExternalAccountKey, ExternalAccountName, ExternalAccountData}` text, `PcCafe/PcCafeCode` text. Parser accepts no XML body. | Builder `0x180001800`; reply parser `0x18000E2B0` |

Connect values above belong to the **game-server library**. Do not use ConnType 401 as a guessed client Connect value. XML field order comes from builders. Reply parser lookup does not establish reply element order.

The consume-token parser permits absent individual fields at its own layer. LobbyDaemon imposes further checks. See [04-token-validation.md](04-token-validation.md).
`LoginWebStart` is a server-library operation. It is not proof that the build-381 password path sends a plaintext Password field.

## Client-authentication names only

The shared `Cli2Auth::CommandToString` function at `0x180028020` names these operations under protocol `Auth`.
PDB enum `Cli2Auth::ECommand` supplies the numeric values. These are not STS library command IDs.

| Name | Enum | Fields and reply pair |
| --- | --- | --- |
| `KeyData` | 22 | ❓ Unknown; cryptographic exchange boundary |
| `LoginCancel` | 23 | ❓ Unknown |
| `LoginFinish` | 24 | ❓ Unknown |
| `LoginStart` | 25 | ❓ Unknown |
| `LoginWebStart` | 26 | Checked server-library body above; ❓ client use |
| `LoginWebFinish` | 27 | ❓ Unknown |
| `RequestCryptKey` | 34 | ❓ Unknown; cryptographic exchange boundary |
| `RequestGameToken` | 35 | ❓ Unknown |
| `RequestSession` | 36 | ❓ Unknown |
| `WaitForAllowedLogin` | 45 | ❓ Unknown |
| `RequestToken` | 46 | ❓ Unknown |
| `LoginTokenStart` | 47 | ❓ Unknown |
| `TokenKeyData` | 48 | ❓ Unknown; cryptographic exchange boundary |

❓ No direct mapping from client symbol `SendLoginRequest` to a wire operation is established.
❓ No direct mapping from `SendTokenRequest` to RequestToken or RequestGameToken is established.
❓ No token-success wire body, token-issue XML fields, or launcher session-key wire body is established.
Do not invent packet fixtures for these unknown bodies.

### PDB API fields, not wire fields

The DLL PDB contains client-side message types even though the inspected server builders do not implement their wire serialization. These names establish API data only. ❓ XML names, optional fields, encodings, and order remain open.

| PDB type suffix after `StsMsgLib::` | API fields |
| --- | --- |
| `_StsMsgRequestLoginStart` | `pszLoginName: char*`, `pszPassWord: char*` |
| `_StsMsgRequestLoginKeyData` | No fields beyond the request base |
| `_StsMsgRequestLoginFinish` | `eLanguageType: ELanguageType` |
| `_StsMsgRequestGameToken` | `pwszGameAlias: wchar_t*` |
| `_StsMsgRequestToken` | `appId: wchar_t*` |
| `_StsMsgRequestLoginTokenStart` | No fields beyond the request base |
| `_StsMsgRequestTokenKeyData` | `isauthnTokenBase64: int`, `authnToken: byte*`, `authnTokenBytes: u32`, `authProviderCode: char*`, `authProviderCodeLen: u32`, `AppId: char*`, `AppIdLen: u32` |
| `_StsMsgResponseLoginStart`, `_StsMsgResponseLoginKeyData`, `_StsMsgResponseTokenKeyData` | No fields beyond the response base |
| `_StsMsgResponseLoginFinish` | `guidUserId: Guid`, `nRoleCount: u32`, `nAuthType: u32`, `nStatus: u32`, `pwszPcCafeCode: wchar_t*`, `pRoleIds: u32*` |
| `_StsMsgResponseRequestGameToken` | `guidGameToken: Guid` |
| `_StsMsgResponseRequestToken` | `authnToken: wchar_t*` |
| `_StsMsgResponseLoginTokenStart` | Public challenge data and a server signature. Cryptographic boundary; see `03-login-flow.md`. |

The distinct response types show that an authentication token string and a game-token GUID are not interchangeable. ❓ Which client call selects each operation remains open. Empty derived types do not prove empty wire bodies.

## Response metadata and errors

`SetResponseMsgInfo` at `0x18000DC40` reads the transaction ID, completion flag, decimal status, and reason phrase. For status >= 400, it reads integer XML root attribute `code`. The root name and complete error schema remain ❓ unchecked.

These three values have different meanings:

| Layer | Checked examples | Source |
| --- | --- | --- |
| STS status | 200; 400 with reason `Success` in LobbyDaemon's consume handler | Existing build-307 spec, `0x1400CECD0` |
| Platform error code | 3352 and 3716 affect presence login | Existing build-307 spec, `0x1400D2310` |
| Library result | Values below | PDB `StsMsgLib::EStsMsgError` |

| Library result | Value |
| --- | --- |
| NONE / SUCCESS / FAIL | 0 / 1 / 2 |
| WRONG_GAME_CODE / WRONG_PARAMETERS | 3 / 4 |
| BUFFER_SIZE_IS_NOT_ENOUGH / WRONG_MESSAGE_BODY | 5 / 6 |
| ALLOC_FAILED / WRONG_KEY_DATA / WRONG_COMMAND | 7 / 8 / 9 |
| ENCRYPT_FAILED / DECRYPT_FAILED | 10 / 11 |

The enum end marker is 12. Do not send library results as platform error codes.

## Wider message library

The DLL also has slot, presence, profile, grade, charge, warehouse, purchase, channel, and promotion builders. This pass does not specify all of them. The existing LobbyDaemon spec covers the login job dependencies. The inventory here is the login/token subset, not a complete NP protocol catalogue.
