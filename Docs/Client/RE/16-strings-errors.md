# 16 — Client.exe 381: error codes, log files, web calls and string anchors

Summary:

1. The client has 4 error code sets: start codes (`ClientSystemText`, 1000–10007, chapter 01), game error codes (`L_ERROR_CODE_TO_STRING` `0x712bc0`, 2,631 names, chapter 07), network codes (`Network::DisconnectReason` 100–154, `DisconnectGameReason` 0–19, `DisconnectLobbyReason` 0–11) and quit-panel states (`QuitGameConfirmPanel::State` 1–44).
2. A lost or kicked session goes `Session::OnError` / `OnClose` → `sub_1129A90(code, a2, a3, state)` (`0x1129a90`) → `QuitGameConfirmPanel::Open` (`0xa1a840`) → `UI.ExitGame.*` text, and at exit `..\log\end.log`.
3. Local log files are in `<bin>\..\log\`: `end.log`, `<date>_<name>_skill.log`, `Log(<date>)-Latency.log`, `Log(<date>)-BattleRoyalTournament.log`. The chat/voice library writes `<dir>/<exe>_<YYYYMMDDhhmm>_<pid>.log`.
4. Remote logs: `ClientLog` sends lines to GameD (`G_WriteClientLogReqCmd` 407). `ClientExecutionLog` and `ClientUserLog` send an encrypted, base64 XML by HTTP POST to `<url>` (rule config `excutionlog-url`, `userlog-url`). Latency goes to GameD (416) and to the web.
5. A crash in `MainFramework` goes to the SEH filter in `WinMain` → `EInterfaceErrorReport` (engine DLL). Client.exe has no own dump writer.

```
 SOCKET / SERVER                      CLIENT                                         SCREEN / FILE
 WSA error, recv/send fail ─► Network::Disconnect 0x5d33c0 (reason 100..154)
                                └► Session::OnError (slot 4)
                                     GameSession 0xf44150  ─ reason +0x60 (DisconnectGameReason)
                                     LobbyGateSession 0xca7080, NPSession 0xc223a0
                                       └► sub_1157DB0 / sub_1157E30 (connection lost, code 1000/2000/3000)
                                       └► sub_1129A90(code, a2, a3, state) 0x1129a90 ─► BNSClient +0x74..+0x80
                                            └► QuitGameConfirmPanel::Open 0xa1a840 ─────────► UI.ExitGame.<state>.Title/Confirm
                                                                                              UI.QuitGame.NetworkErr / SystemErr / WSALastErr
                                            └► QuitGameConfirmPanel::endProgress 0xa15f20 ─► ..\log\end.log
 server error code (u16) ─► L_ERROR_CODE_TO_STRING 0x712bc0 (table 0x194fc30) ─► log text; feature handlers ─► Msg.* alias
 ClientLog::WriteV 0x10dcd50 ─► 3-line queue ─► ClientLog::Flush 0x10dc8e0 ─► G_WriteClientLogReqCmd 407 ─► GameD
 ClientExecutionLog::SendLog 0x11675c0 / ClientUserLog::SendLog 0x10e90d0 ─► thread ─► HTTPClient::Request ─► POST <url>
 WinMain __except ─► EInterfaceErrorReport (engine DLL, ptr 0x1a4c0e0)
```

Rules: "381" = `Client.exe` 0.0.381 (x86). "419 GM" = names and types from the 419 GM PDB (types are in the 381 til).
❓ marks a fact or a name that we did not verify. URLs are written `<url>`. Keys are written `keyhere`.

Files of this chapter:

| File | Rows | Content |
| --- | ---: | --- |
| [`16-errors-enums-gm.tsv`](16-errors-enums-gm.tsv) | 3,478 | All values of 232 enums with `Disconnect`, `Reason`, `Error`, `Err`, `Fail`, `Kick`, `Exception`, `Log` in the name (419 GM types in the 381 til) |
| [`16-ruleconfig-options.tsv`](16-ruleconfig-options.tsv) | 701 | Rule-config option table `0x19dde64`–`0x19e40d4`: key, type, 381 field offset, 419 GM field name, default, web/log flag |
| [`16-strings-errors-functions.tsv`](16-strings-errors-functions.tsv) | see file | Functions of the log, web and error classes with 381 addresses |
| [`16-string-anchors.tsv`](16-string-anchors.tsv) | see file | String anchors per subsystem (§3) |

Related chapters: start errors [01 §3.7](01-startup.md#37-start-errors), socket disconnect [02 §5](02-transport.md#5-disconnect-timeouts-and-reconnect), STS login errors [03 §5–6](03-sts-client.md#5-errors-shown-to-the-user), game error-code table [07 §4.2](07-combat.md), quest errors [`10-quests-errors.tsv`](10-quests-errors.tsv), UI [14](14-ui.md).

---

## Quick find

| Task or question | Function / item | 381 address | GM name | Section |
| --- | --- | --- | --- | --- |
| Error code → name text | `L_ERROR_CODE_TO_STRING` (table `0x194fc30`) | `0x712bc0` | same | §1.2 |
| Start error code → `SystemText.ini` text | `ClientSystemText::SetAdditionalMsg` | `0x10a0110` | same | §1.1 |
| Where GameD disconnect reason picks the quit panel | `GameSession::OnError` | `0xf44150` | same | §1.4 |
| Where LobbyGate / NP lost picks the panel | `sub_1157E30` | `0x1157e30` | ❓ | §1.4 |
| Where GameD lost picks the panel | `sub_1157DB0` | `0x1157db0` | ❓ | §1.4 |
| Set quit state + codes | `sub_1129A90` (writes `BNSClient` `+0x74..+0x80`) | `0x1129a90` | ❓ | §1.4 |
| Quit panel text per state | `QuitGameConfirmPanel::Open` | `0xa1a840` | same | §1.5 |
| `end.log` at exit | `QuitGameConfirmPanel::endProgress` | `0xa15f20` | same | §2.1 |
| WSA error shown on the panel | `BNSClient::GetDisconnectWSALastError` | `0x520c60` | same | §1.5 |
| Client log line → GameD | `ClientLog::WriteV` → `ClientLog::Flush` | `0x10dcd50`, `0x10dc8e0` | same | §2.3 |
| Execution log POST | `ClientExecutionLog::SendPostArguments` | `0x1167300` | same | §2.4 |
| User log POST | `ClientUserLog::SendPostArguments` | `0x10e8df0` | same | §2.4 |
| Skill trace file | `SkillTracer::Report` | `0x96bcf0` | same | §2.1 |
| Latency log file | `TaskSession::InitializeLatencyLogModule` | `0x876ed0` | same | §2.1 |
| Generic log file object | `LogFile` ctor (IDA name `PvpResultDeadProperty::PvpResultDeadProperty` is false) | `0x5d0da0` | ❓ | §2.2 |
| Crash filter | `WinMain` `__except` → `EInterfaceErrorReport` | `0x119104f` | same | §2.6 |
| WinINet wrapper | `HTTPClient::connect` | `0xea2148` area | same | §2.5 |
| Rule-config option table | first entry `region` | `0x19dde64` | — | §2.7 |

---

## 1. Error codes and messages

### 1.1 Start codes (`ClientSystemText`)

`ClientSystemText::SetAdditionalMsg(code, text)` (`0x10a0110`) has 25 call sites. All are in [`01-startup-error-codes.tsv`](01-startup-error-codes.tsv) (we checked the call list again: same 25 sites). The text comes from `..\bin\SystemText.ini` (chapter 01 §3.7). Other `ClientSystemText` functions: `read` `0x10a0180`, `ResetSystemText` `0x10a02d0`, `SetFile` `0x10a4df0`, `GetSystemText` `0x10b75d0`, `readAll` `0x10b77d0`, `ReadAll` `0x10b7bf0`, ctor `0x10c9040`, dtor `0x10c8f80`.

| Range | Area |
| --- | --- |
| 1000–1002 | XML reader DLL and `xml.dat` |
| 2000–2002 | engine DLL load and version |
| 3000, 3002 | client expire / start checks |
| 4000–4050 | GameGuard (out of scope; only the code list) |
| 5000–5002 | zone map load (`UiTransitControl::loadZoneMap` `0x790344`…) |
| 6000 | Tencent TCLS |
| 10001–10007 | XML data and locale checks |

### 1.2 Game error codes (server → client)

The server sends a 16-bit error code in many `*FailCmd` / `*Ack` packets. `L_ERROR_CODE_TO_STRING` (`0x712bc0`) gives the name from the pointer table `0x194fc30` (codes 0–0xA46). The full list (2,631 names) is [`07-combat-error-codes.tsv`](07-combat-error-codes.tsv). Each feature handler picks its own `Msg.*` alias for a code (for example `GoodsStorePurchaseConfirmPanel::getPurchaseErrorByErrorCode` `0xedf5f0`, `BParty::InvokePartyErrorMessage` `0xc4b380`, `BTeam::InvokeTeamErrorMessage` `0x8a5710`, `Guild2BankPanel::ProcessErrorMessage` `0xf107a0`, `ArenaDuelLobby::invokeErrorMessage` `0x125d870`, `ArenaDungeonLobby::invokeErrorMessage` `0x12754c0`). There is no single code → alias table ❓ (we found none).

### 1.3 Network and session reason enums (419 GM, used by 381 code)

These close the ❓ of [02 §8](02-transport.md#8-enums-419-gm-names-values-used-in-381-code).

| Enum | Values |
| --- | --- |
| `Network::Error` | 0 NE_NO_ERROR, 1 NE_UNKNOWN_ERROR, 2 NE_CONNECTION_IS_NULL, 3 NE_CONNECTION_IS_EXISTS, 4 NE_MEMORY_ERROR, 5 NE_CONNECT_FAILED, 6 NE_SEND_FAILED |
| `Network::DisconnectReason` | 100 NORMAL, 101 TIMEOUT, 102 NET_ERROR, 103 NET_ERROR_INVALID_SESSION, 104 NET_ERROR_CONN_RESET, 105 DATA_ERROR, 130 RECV_FAIL_COMMON, 131 RECV_FAIL_DISCONNECTED, 132 RECV_FAIL_OVERLAPPEDFAIL, 133 RECV_FAIL_RESULT_ZERO, 134 RECV_FAIL_INVALID_PACKET, 135 RECV_FAIL_INVALID_PACKET2, 136 RECV_FAIL_DECRYPT_FAIL, 150 SEND_FAIL_COMMON, 151 SEND_FAIL_DISCONNECTED, 152 SEND_FAIL_OVERLAPPEDFAIL, 153 SEND_FAIL_RESULT_ZERO, 154 SEND_FAIL_SEND_NEXT_FAIL (prefix `DISCONNECT_` / `DISCONNECT_NET_ERROR_`) |
| `Connection::SendErr` | 0 SEND_ERR_NONE, 1 DISCONNECTED, 2 OVERLAPPEDFAIL, 3 RESULT_ZERO, 4 SEND_NEXT_FAIL (prefix `SEND_COMPLETE_ERR_`) |
| `DataProcessor::ErrCode` | 0 DP_NO_ERROR, 1 INVALID_PARAM, 2 NOT_SUPPORT, 3 ALLOCATE_FAILED, 4 INSUFFICIENT_BUFFER, 5 READFILE_FAILED, 6 WRITEFILE_FAIELD, 7 NOT_READY_CRYPTO, 8 ALREADY_CRYPTO, 9 CRYPTO_FAILED, 10 ENCRYPT_FAILED, 11 DECRYPT_FAILED |
| `DisconnectGameReason` (GameSession `+0x60`) | 0 NONE, 1 DISCONNECTED_NP, 2 DISCONNECTED_LOBBYGATE, 3 KICK_DUPLICATE_LOGIN, 4 KICK_CREDENTIAL_CHANGED, 5 KICK_BY_GM, 6 KICK_BY_SYSTEM, 7 KICK_BY_XIGNCODE, 8 NO_RESPONSE_SERVER_PING, 9 INVALID_RESPONSE_SERVER_PING, 10 SEND_BUFFER_OVERFLOW, 11 TOO_MANY_RECEIVED_PACKET, 12 DISCONNECTED_ARENA, 13 MOVEFAIL_ARENA, 14 MOVEFAIL_DUEL, 15 RETURN_TO_LOBBY, 16 EXCEED_SECOND_PASSWORD_TRY_LIMIT, 17 KICK_BY_SHUTDOWN, 18 STOPPLAY_BY_SHUTDOWN, 19 MOVEFAIL_BATTLEROYAL |
| `DisconnectLobbyReason` | 0 NONE, 1 DISCONNECTED_GAME, 2 DUPLICATE_LOGIN_KICKED_LOBBYGATE, 3 CREDENTIAL_CHANGED_KICKED_LOBBYGATE, 4 EXCEED_SECOND_PASSWORD_TRY_LIMIT, 5 SYSTEM_KICKED_LOBBYGATE, 6 CONNECT_FAIL_LOBBYGATE, 7 DUPLICATE_LOGIN_KICKED_NP, 8 CREDENTIAL_CHANGED_KICKED_NP, 9 SYSTEM_KICKED_NP, 10 LOGIN_FAIL, 11 KICK_BY_SHUTDOWN |
| `Log::Type` | 0 LT_ERROR, 1 LT_DEBUG, 2 LT_TRACE, 3 LT_PACKET, 4 LT_CRITICAL, 5 LT_MAX |
| `DrLog::Type` | 0 Trace, 1 Debug, 2 Error |
| `USERLOG_TYPE` | 0 NONE, 1 UIHISTORY, 10003 ENTERWORLD_ONCE, 11201 DPS, 17001 NP_CUSTOMIZEDATA_ENCODING_ERROR, 17002 ITEM_GRAPH_PATH_FINDING_SUCCESS, 17003 CUSTOMCHANGER_CONFIG, 17004 CLIENT_INITIALIZE_FAILED, 17005 MAX |

All other error, fail and reason enums (232 enums, for example `CRITICALLOG_TYPE` with 282 values, `BParty::PartyCheckErr`, `CheckUsableNameErr`) are in [`16-errors-enums-gm.tsv`](16-errors-enums-gm.tsv).

### 1.4 Session error → quit state

`sub_1129A90` (`0x1129a90`, `__usercall`: `eax` = code, `ecx` = a2, `esi` = a3, stack = state) writes `BNSClient` `+0x74` = state, `+0x78` = a3, `+0x7C` = code, `+0x80` = a2. Then it sets the UI to a modal state (`+4` = 17) and calls the `ShowQuitGameConfirm` of the active UI (`sub_7E41D0`, `sub_7E3D00`, `sub_752330`, `BUiArenaLobby::ShowQuitGameConfirm`, `BUiGuildCustomize::ShowQuitGameConfirm`).

`GameSession::OnError` (`0xf44150`). Guard: skip when the session state `+0x34` = 8 and the reason = 12 (arena move). Else clear the server-ping vector, clear `+0x5C`/`+0x5D`, and switch on `DisconnectGameReason` (`+0x60`):

| `DisconnectGameReason` | Quit state | Call site |
| --- | --- | --- |
| 3 KICK_DUPLICATE_LOGIN | 12 DUPLICATE_LOGIN | `0xf441b2` |
| 4 KICK_CREDENTIAL_CHANGED | 13 CREDENTIAL_CHANGED | `0xf441cb` |
| 5 KICK_BY_GM | 16 KICK_BY_GM | `0xf441e4` |
| 7 KICK_BY_XIGNCODE | 23 KICK_BY_XIGNCODE | `0xf441fd` |
| 8 NO_RESPONSE_SERVER_PING | 17 KICK_NO_RESPONSE_SERVER_PING | `0xf44216` |
| 9 INVALID_RESPONSE_SERVER_PING | 18 KICK_INVALID_RESPONSE_SERVER_PING | `0xf4422f` |
| 10 SEND_BUFFER_OVERFLOW | 19 KICK_SEND_BUFFER_OVERFLOW | `0xf44248` |
| 11 TOO_MANY_RECEIVED_PACKET | 20 KICK_TOO_MANY_RECEIVED_PACKET | `0xf44261` |
| 16 EXCEED_SECOND_PASSWORD_TRY_LIMIT | 21 KICK_EXCEED_SECOND_PASSWORD_TRY_LIMIT | `0xf4427a` |
| 17 KICK_BY_SHUTDOWN | 22 KICK_SHUTDOWN | `0xf44293` |
| other (0, 1, 2, 6, 12–15, 18, 19) | `sub_1157DB0(kind, a2, a3)` (`0xf442ae`) | |

`sub_1157DB0` (`0x1157db0`, GameD): kind 3 (lost) → `sub_1129A90(1000, a3, a2, 8 CONNECTION_LOST)`. Kind 2 with a2 = 5 (`NE_CONNECT_FAILED`) → post UI event 27 to the UI at `BNSClient +0x48` ❓.
`sub_1157E30` (`0x1157e30`, LobbyGate and NP; called from `NPSession::OnError` `0xc224d5` and `LobbyGateSession::OnError` `0xca709a`, `0xca70d6`): kind 3 → `sub_1129A90(3000 if a4 else 2000, a3, a2, 8)`. Kind 2 with a2 = 5 → store a3 in `BNSClient +0x80` and call `BUiLobby::OnConnectFailed`.

❓ We think kind = 2 "connect result" and 3 "disconnect", a2 = `Network::Error` / `DisconnectReason`, a3 = WSA error. We did not trace the `Network` code that calls slot 4 with these values.

Other callers of `sub_1129A90`:

| Site | Function | State |
| --- | --- | --- |
| `0x74ced7` | `World::Tick` | 11 LOGOUT_BY_SERVER_COOLDOWN |
| `0xbc1240` | `BPlayer::ProcessNetObjectEvent` | 44 INVALID_GAME_PACKET_RIGHT |
| `0xc224f4` | `NPSession::OnError` | 12 DUPLICATE_LOGIN |
| `0xc24d69` | `NPSession::Tick` | 8 CONNECTION_LOST (reconnect gave up, chapter 03 §6) |
| `0xc43f37`, `0xc43f49`, `0xc43f5e` | `NPSession::OnClose` | 12, 13, 8 |
| `0xc672c6` | `sub_C67290` (`LobbyGateSession::OnClose`, 02 §1.2) | 22 KICK_SHUTDOWN |
| `0xc87b56` | `sub_C87B00` | 8 |
| `0xc97947`, `0xc9796f` (+4 calls with a register state) | `LobbyGateSession::ReturnToLobby` | 8 / other |
| `0xcb87df`, `0xcb8850` | `LobbyGateSession::ProcessLobbyNetObjectEvent` | 8 |
| `0xefbe3c` | `GameSession::OnClose` | 22 KICK_SHUTDOWN |
| `0xf4b415`, `0xf4e166`, `0xf4e1ac`, `0xf4e1ea` | (no function) | 8 |
| `0xf4e181` | (no function) | 34 ENTERWORLDFAIL_BY_CHARGE |
| `0x1157de3`, `0x1157e90` | `sub_1157DB0`, `sub_1157E30` | 8 |

### 1.5 Quit panel states and text

`QuitGameConfirmPanel::State` (419 GM): 1 EXIT_GAME, 2 EXIT_GAME_TO_RESTART_LAUNCHER, 3 TO_LOBBY, 4 TO_TITLE, 5 TO_LOBBY_LIGHTCLIENT, 6 TO_LOBBY_LEVEL_DOWNLOAD, 7 VERIFY_OTP_FAIL, 8 CONNECTION_LOST, 9 MOVEFAIL_ARENA, 10 MOVEFAIL_DUEL, 11 LOGOUT_BY_SERVER_COOLDOWN, 12 DUPLICATE_LOGIN, 13 CREDENTIAL_CHANGED, 14 LOGIN_FAIL, 15 LOGIN_FAIL_CONSECUTIVE, 16 KICK_BY_GM, 17 KICK_NO_RESPONSE_SERVER_PING, 18 KICK_INVALID_RESPONSE_SERVER_PING, 19 KICK_SEND_BUFFER_OVERFLOW, 20 KICK_TOO_MANY_RECEIVED_PACKET, 21 KICK_EXCEED_SECOND_PASSWORD_TRY_LIMIT, 22 KICK_SHUTDOWN, 23 KICK_BY_XIGNCODE, 24 KICK_BY_HEALTHY_SYSTEM, 25 SEND_DELIVERY, 26 RECEIVE_DELIVERY, 27 RECEIVE_DELIVERY_STACK, 28 REGISTER_PRODUCTION, 29 REGISTER_AUCTION_SALES, 30 REGISTER_AUCTION_BID, 31 UPDATE_AUCTION_BID, 32 REGISTER_AUCTION_PURCHASE, 33 RECEIVE_STOP_PLAY, 34 ENTERWORLDFAIL_BY_CHARGE, 35 EXCEPTION_ARENA_LOBBY, 36–39 ITEM_CLEANING_DISPOSE/SELL/DECOMPOSE/EXCHANGE, 40 INGAMESHOP_CHARGE, 41 ACHIEVEMENT_REWARD, 42 JUMPING_TO_LOBBY, 43 MOVEFAIL_BATTLEROYAL, 44 INVALID_GAME_PACKET_RIGHT (prefix `STATE_`, `STATE_PROGRESS_` for 1–6).

`QuitGameConfirmPanel::Open` (`0xa1a840`) picks the text alias by state. Panel fields: `+0x6C` state, `+0x70` second value (network reason ❓), `+0x74` third value.

| State | Title alias | Confirm alias | Site |
| --- | --- | --- | --- |
| 1 | `UI.ExitGame.Title` | `UI.ExitGame.Confirm` (+ `.Event.RandomStore`, `.Event.Attendance`, `.Event.ChallengeToday`, `.EventGuide`) | `0xa1af00` |
| 2 | `UI.ExitGame.Title` | `UI.RestartLauncher.Confirm` | `0xa1b441` |
| 3 | `UI.ExitGame.ToLobby.Title` | `UI.ExitGame.ToLobby.Confirm` | `0xa1b49d` |
| 6 | | `UI.ExitGame.ToLobby.LevelDownload.Confirm` | `0xa1b56f` |
| 4 | `UI.ExitGame.ToTitle.Title` | `UI.ExitGame.ToTitle.Confirm` | `0xa1b5bf` |
| 7 | `UI.ExitGame.OtpVerifyFail.Title` | `UI.ExitGame.OtpVerifyFail.Confirm` | `0xa1b600` |
| 8 | `UI.ExitGame.DisConnect.Title` | `UI.ExitGame.DisConnect.Confirm` | `0xa1b63c` |
| 9 | `UI.ExitGame.MoveFailArena.Title` | `UI.ExitGame.MoveFailArena.Confirm` | `0xa1b66c` |
| 10 | `UI.ExitGame.MoveFailDuel.Title` | `UI.ExitGame.MoveFailDuel.Confirm` | `0xa1b6b4` |
| 43 | | `UI.ExitGame.MoveFailBattleRoyal.Confirm` | `0xa1b6e4` |
| 11 | `UI.ExitGame.ByServerCooldown.Title` | `UI.ExitGame.ByServerCooldown.Confirm` | `0xa1b709` |
| 12 | `UI.ExitGame.DuplicateLogin.Title` | `UI.ExitGame.DuplicateLogin.Confirm` | `0xa1b73b` |
| 13 | `UI.ExitGame.CredentailChanged.Title` (sic) | `UI.ExitGame.CredentialChanged.Confirm` | `0xa1b777` |
| 16 | `UI.ExitGame.KickByGM.Title` | `UI.ExitGame.KickByGM.Confirm` | `0xa1b7b3` |
| 17 | `UI.ExitGame.Kick.NoRespopnseServerPing.Title` (sic) | `….Confirm` | `0xa1b7ef` |
| 18 | `UI.ExitGame.Kick.InvalidResponseServerPing.Title` | `….Confirm` | `0xa1b82b` |
| 19 | `UI.ExitGame.Kick.SendBufferOverFlow.Title` | `….Confirm` | `0xa1b867` |
| 20 | `UI.ExitGame.Kick.TooManyReceivedPacket.Title` | `….Confirm` | `0xa1b8a3` |
| 22 | `UI.ExitGame.Kick.Shutdown.Title` | `UI.ExitGame.Kick.Shutdown.Confirm` (shutdown time text) | `0xa1b8ec` |
| 23 | `UI.ExitGame.Kick.XignCode.Title` | `UI.ExitGame.Kick.XignCode.Confirm` | `0xa1b975` |
| 21 | `UI.ExitGame.Kick.ExceedSecondPasswordTryLimit.Title` | `….Confirm` | `0xa1b9a7` |
| 33 | `UI.ExitGame.ChargeExpireStopPlay.Title` / `UI.ExitGame.FinishedTrialVersion.Title` / `Kick.Shutdown` (by sub-value) | `….Confirm` | `0xa1be21`–`0xa1bedd` |
| 34 | `UI.ExitGame.EnterWorldFailByCharge.Title` | `.ChargeInfoNotFound`, `.ChargeExpired`, `.ChargeAlreadyStarted`, `.ChargeDifferentIpKind`, `.ChargeSuspended`, `.ChargeConFlictIp` `.Confirm` (sub-value 0x796–0x79A) | `0xa1bf47`–`0xa1bfca` |
| 35 | `UI.ExitGame.ToLobby.Title` | `UI.InterserverTotalDungeon.Confirm.EnterFail.InvalidArenaPortal` / `UI.InterserverCombineDungeon…` | `0xa1bff2`–`0xa1c06c` |
| 44 | `UI.ExitGame.Kick.InvalidGamePacketRight.Title` | `….Confirm` | `0xa1c0bf` |
| 25–32, 36–41 | progress panels: `UI.SendingDeliery.Progress.*`, `UI.QuickReceivingDeliery.Progress.*`, `UI.RegisteringProduction.Progress.*`, `UI.RegisteringAuction{Sales,Bid,Purchase}.Progress.*`, `UI.UpdatingAuctionBid.Progress.*`, `UI.ItemCleaning.{Dispose,Sell,Decompose,Exchange}.Progress.*`, `UI.IngameShop.Charge.Progress.*`, `UI.Achievement.Reward.Progress.*2` (Title, InProgress, Failed, TimeOut) | | `0xa1ab85`–`0xa1ae0b` |

Error lines added under the text (all states in the error group):
- `UI.QuitGame.NetworkErr` (`0x1634770`, site `0xa1c2e7`) with panel `+0x70`, only when state = 8 and `+0x70` ≠ 0.
- `UI.QuitGame.SystemErr` (`0x1633e1c`, sites `0xa1bb2d`, `0xa1c1ed`) with panel `+0x74` and a text.
- `UI.QuitGame.WSALastErr` (`0x1633e48`, sites `0xa1bb63`, `0xa1c31d`) with `BNSClient::GetDisconnectWSALastError` (`0x520c60`).

---

## 2. Log files, web logs and crash path

### 2.1 Local files

`<bin>` = the wide string at `0x1a4c100` (folder of `Client.exe` ❓; filled at start, used by 10+ functions).

| File | Writer | 381 address | Content |
| --- | --- | --- | --- |
| `<bin>\..\log\end.log` | `QuitGameConfirmPanel::endProgress` | `0xa15f20` (path `0x16347a0`, name `0x16347b4`) | `CREATE_ALWAYS`, UTF-16 with BOM `0xFEFF`. One line `%d-%d-%d %d:%d:%d, %d, %d, %d` (`0x16347c4`) = local time, panel `+0x6C` (quit state), `+0x70`, `+0x74`. Then the options are saved (`BUiWorld::WriteOption`, `BUiArenaLobby::WriteOption`, `UiStateGuildCustomize::WriteOption`). |
| `<bin>\..\log\<Y-M-D_h-m-s_><name>_skill.log` | `SkillTracer::Report` | `0x96bcf0` (fmt `0x1650b08`, suffix `0x1650b30`) | Skill trace summary line `name:%s, geoZoneId:%d, total:%d(%lluKB), max:%d(%lluKB), success:%d, lateResponse(over %llums):%d, noResponse:%d, fail:%d, clientFail:%d, serverFail:%d, minLatency…, maxLatency…, averageLatency…` (`0x1650958`). Config: `[SkillTrace]` keys (chapter 01 §2.1, `SkillTracer::LoadIni` `0x906b80`). |
| `<bin>\..\log\Log(<%Y%m%d_%H%M%S>)-Latency.log` | `TaskSession::InitializeLatencyLogModule` → `LogFile` | `0x876ed0` | Network latency log (module flag set in the `LogFile`) |
| `<bin>\..\log\Log(<%Y%m%d_%H%M%S>)-BattleRoyalTournament.log` | `BattleRoyalField::initLog` → `LogFile` | `0x119de90` | Battle royal log, module index 131 enabled |
| `<dir>/<exe>_<YYYYMMDDhhmm>_<pid %06d>.log` | `LogManager::Start` / `CreateLogFile` | `0x5e6470`, `0x5e6630` (fmt `0x186d3d0`) | Chat / voice client library (`ClientManager::Initialize` `0x5ecc21`, `VC::ClientManager::Initialize` `0x604b70`). Opens with `OPEN_ALWAYS`, appends, writes a BOM. `<exe>` = module name without extension. Optional writer thread (`LogThreadProc` `0x5e6610`/`0x5e6790`) and console echo. |
| `.\frostStatic.log` | `sub_633590` | `0x633590` (string `0x139d978`) | 3rd-party library static log ❓ |
| `kill.log` (`0x1650b34`) | none (no xref) | — | ❓ unused string |

### 2.2 `LogFile` object (`0x5d0da0`, size 0x250)

`LogFile` ctor (IDA name is false) → `sub_5D0F20` (`0x5d0f20`). Fields: `+0x230` (dword 140) level, clamp 0..4; `+0x240` buffer size, 4,096 if 0, 0 if level is 0; `+0x234` file path. Path: dir + `\` + `Log(%s)-%s.log` (`0x186c628`, `%s` = `wcsftime("%Y%m%d_%H%M%S")` `0x17c3f7c`, then the module name) or `Log(%s).log` (`0x186c64c`) when the name is empty. The buffer grows ×2 from 4,096 to the asked size. `LogFile::write` `0x5d12d0`, `LogFile::writeFile` `0x5d1470`. Module enable array `moduleEnabled[]` at `+0x0D`, 512 bytes; index = log module number ([`01-startup-log-modules.tsv`](01-startup-log-modules.tsv)).

Note: the IDA name `Log::Write` (`0x5d2150`) is false for most callers: callers use it as "get allocator" (`+4` = alloc, `+0xC` = free).

### 2.3 `ClientLog` → GameD

| Step | Function | 381 address | Behaviour |
| --- | --- | --- | --- |
| 1 | `ClientLog::Initialize` | `0x10dce00` | From `BNSClient::Initialize` (`0x116f6ae`). Allocates 0x28 bytes; global `0x1a4c464`. |
| 2 | `ClientLog::WriteV(file, line, module, logId, objId, …, fmt, args)` | `0x10dcd50` | `GetLocalTime`, `vswprintf` into 1,999 chars, calls `write`. |
| 3 | `ClientLog::write` | `0x10dc9d0` | Counts lines per text in a map (`+4`). Formats `%04d/%02d/%02d %02d:%02d:%02d.%03d, %s, %d, %s, %d, %s\r\n` (`0x151cce8`: time, file, line, module name, …, text). Keeps the queue at `+0x14` at 3 lines or fewer (drops the oldest). Pushes the line and calls `Flush`. |
| 4 | `ClientLog::Flush` | `0x10dc8e0` | Only if enabled (`+0x10` ≠ 0) and the GameD session is connected. For each queued line: build `G_WriteClientLogReqCmd` 407 (vtable `0x179c9d0`, id `0x197`), field `log` (`PfString`), `BNSClient::SendToGamed`. Stops at the first send that fails. |
| — | Reply | `G_WriteClientLogOkCmd` 408 (`0xee10c0`), `G_WriteClientLogFailCmd` 409 (`failReason` i16, `0xee10b0`), `G_RequestClientLogNotifyCmd` 410 (`type` i8, `0xef64d0` in `World::ProcessNetObjectEvent`) | ❓ We did not find the code that sets `+0x10`; we think 410 turns the upload on. |

Packet layouts: [`packet-layouts.tsv`](packet-layouts.tsv) rows 407–410, 416. `G_WriteLatencyLogReqCmd` 416 (`TaskSession::requestReportLatency` `0x8b7fe0`): `latencyType` i8 @16, `min` i32 @20, `max` i32 @24, `avg` i32 @28, `count` i32 @32, `privateIP` i32 @36 (40 bytes).

### 2.4 Web logs (HTTP POST)

| Item | `ClientExecutionLog` | `ClientUserLog` |
| --- | --- | --- |
| Start | `Startup` `0x1131030` (from `BNSClient::Initialize`, chapter 01 §3.4): exe path and name | `FormattedWrite(text, ?, ?, USERLOG_TYPE)` `0x10eddd0` |
| Data | `AddSystemInfo` `0x1131000`, `AppendOSName` `0x11228d0`, `AppendBaseDirName` `0x1122a70`, `AppendBinaryName` `0x1122ac0`, `AppendCommandLine` `0x11291f0`, `AppendBinaryVersion` `0x1129370`, `AppendServerIP` `0x1129420`, `AppendMachineIP` `0x1130ed0` → `MakeLogXmlFile` `0x11294e0` | `MakeLogString` `0x10c8900`, `MakeLogXmlString` `0x10aac80`, `GetUserLogFieldValue` `0x10c6f50` |
| Send | `SendLog` `0x11675c0`: thread `SendPostArguments` `0x1167300`, waits on an event | `SendLog` `0x10e90d0`: thread `SendPostArguments` `0x10e8df0`, waits on an event |
| Crypto | `DataProcessor::Create(4)`, 16-byte key at `0x13f7630` (`keyhere`), block cipher, then `base64_encode` | same key and steps |
| Form field | `ExecutionLog` | `UserLog` |
| URL | `GRuleConfig +0x14` `excutionlog-url` (`+0x18` `excutionlog-url2` when language = 10 VIETNAMESE) | `+0x20` `userlog-url` (`+0x24` `userlog-url2` for language 10); on only if `+0x1C` `use-userlog` |
| Built-in default | `<url>` (fixed IP, path `ExecutionLog/default.aspx`, `0x17803e0`) | `<url>` (fixed IP, path `UserLog/default.aspx`, `0x1780388`) |
| Request | `HTTPClient::Request(url, POST (2), …)` | same |

`USERLOG_TYPE` callers of `FormattedWrite`: `World::EnterWorld` (`0x741d01`, `0x7428fb`, ENTERWORLD_ONCE), `NPSession::SendCustomizeDataSaveRequest` (`0xc2703e`, NP_CUSTOMIZEDATA_ENCODING_ERROR), `ItemGraphPanel::refreshPathFinder` (`0xdd5ec5`), `DamageMeterPanel::sendBattleCriticalLog` (`0x1079864`, DPS) ❓ type per site not read.

### 2.5 HTTP client (WinINet)

`HTTPClient` uses `InternetOpenW` (`0xea2148`) and `InternetConnectW` (`0xea21ca`) in `HTTPClient::connect`. Error log strings (UTF-16, `0x159a6e8`–`0x159b870`): `HttpOpenRequestW uri(%s), last-error(%u)`, `HttpAddRequestHeadersW accept(%s)…`, `HttpSendRequestW uri(%s), last-error(%u), time(%d)`, `HttpSendRequestEx(multi)…`, `HttpEndRequest(multi)…`, `failed parse response. NULL _httpRequest`, `http query info error(%u), infoLevel(%u), ERROR_HTTP_HEADER_NOT_FOUND`, `failed HttpOpenRequestW(POST)…`. Source path string `0x159a270` (`…\Source\ClientEXE\Private\H…`). Function list: [`16-strings-errors-functions.tsv`](16-strings-errors-functions.tsv).

### 2.6 Crash path

`WinMain` (`0x1190f60`) calls `BNSClient::MainFramework` inside `__try` (`0x1191033`). The `__except` filter (`0x119104f`) calls `EInterfaceErrorReport(exception pointers)` and returns its value. The pointer is at `0x1a4c0e0`; `LoadEngine` sets it (`GetProcAddress`, `0x114c959`); `UnloadEngine` clears it (`0x1122fdb`). Client.exe imports no `MiniDumpWriteDump`. `SetUnhandledExceptionFilter` is used only by the CRT (`___report_gsfailure`, `___CxxSetUnhandledExceptionFilter`) and GameGuard (`_PreInitNPGameMonA`). So the crash report is in `bsengine_Shipping.dll` ❓ (not read). `ReportEventW` is used only by `_OPENSSL_showfatal` (`0x66b8d2`).

### 2.7 Rule-config web and log options

The rule-config option table is at `0x19dde64`–`0x19e40d4` (701 entries of 0x24 bytes). Entry: `+0x08` index, `+0x0C` field offset in `ClientRuleConfig` (`GRuleConfig` pointer at `0x194f658`), `+0x10` key (UTF-16), `+0x14` type (1 int, 3 float, 4 bool, 6 string, 9 group), `+0x1C` default, `+0x20` description. 381 field offsets are not the same as 419 GM after about `0x2f0` (for example `print-network-log` 381 `0x2f9`, 419 GM `0x2fd`). All rows: [`16-ruleconfig-options.tsv`](16-ruleconfig-options.tsv), column `web_or_log` = 1 for 238 web, log and report rows.

| Key | 381 offset | Type | Use |
| --- | --- | --- | --- |
| `excutionlog-url`, `excutionlog-url2` | `0x14`, `0x18` | string | §2.4 |
| `use-userlog`, `userlog-url`, `userlog-url2` | `0x1C`, `0x20`, `0x24` | bool, string | §2.4 |
| `uilog-all-use`, `uilog-world-id` | `0x28`, `0x2C` | bool, int | UI history log (`USERLOG_TYPE_UIHISTORY`) ❓ |
| `print-network-log` | `0x2F9` | bool | network latency group |
| `send-network-latency-period` | `0x2FC` | int | |
| `send-ping-period` | `0x300` | int | |
| `bad-ping-tolerance` | `0x304` | int (ms) | "tolerance for bad ping immediete report" |
| `send-network-latency-to-gamed` | `0x308` | bool | 416 to GameD |
| `send-network-latency-to-web` | `0x309` | bool | web report |
| `use-gamelog`, `gamelog-url`, `gamelog-home-urn` | `0x465`, `0x468`, `0x46C` | bool, string | in-game web game log page |
| `use-play-report`, `use-report-bot-spammer`, `use-report-friend-proposed-spammer` | `0xF5`–`0xF7` | bool | report features |
| `use-client-spammer-report`, `use-new-client-spam-report`, `client-spammer-report-modulate` (10) | `0x1AA`–`0x1AC` | bool, int | spam report |

### 2.8 Network latency report

| Step | Function | 381 address | Behaviour |
| --- | --- | --- | --- |
| Tick | `TaskSession::Tick` | `0x8d69d0` | Ping check with `requestRoundTripTimeForPingCheck` (`0x8b8080`) when `GRuleConfig +0x300` `send-ping-period` > 0 and the timer passes. Writes the latency log (`writeNetworkLatencyLog` `0x8810e0`) when the latency log flag (`0x1a4c329`) is set. Calls `reportNetStateToServer` when the `+0x2FC` period passes. |
| Server report | `TaskSession::reportNetStateToServer` | `0x8cca00` | Only if `GRuleConfig +0x308` `send-network-latency-to-gamed` ≠ 0. Sends 2 × `requestReportLatency` (type, min, max, avg = sum / count), then resets min = 0xFFFF, max = 0, sum = 0, count = 0. |
| Packet | `TaskSession::requestReportLatency` | `0x8b7fe0` | `G_WriteLatencyLogReqCmd` 416 (§2.3) |
| Log file | `TaskSession::InitializeLatencyLogModule` | `0x876ed0` | §2.1 |
| Print flag | `sub_880E50`, `BNSClient::Initialize` | `0x880ee7`, `0x1170780` | read `+0x2F9` `print-network-log` |

❓ We found no reader of `+0x309` `send-network-latency-to-web` in 381.

### 2.9 Other log writers

| Writer | 381 address | Format / anchor |
| --- | --- | --- |
| `LOG_LIB::LogFormattingW` | `0x5e69c0` | `[%9s] %s (%d) : ` (`0x186d438`) — chat / voice library |
| `LOG_LIB::LogFormattingA` | `0x5e6a40` | `[%9s] %s (%d): ` (`0x186d45c`) |
| `Network::SocketError` | `0x5d2930` | `%d; %s; %s` (02 §4.4) |
| `BNSClient::Tick` | `0x1167630` | `[Engine] Critical Log: %s` (`0x14fdf04`), `exit by security. (%s)` (`0x14fdf38`) |
| `WebSessionManager::SessionLog::ToString` | `0x723750` | `%02d-%02d:%02d:%02d,%c,%d,%u,%s` (`0x16afda0`); `SendAndClearSessionHistoryLog` `0x736b00`, `AddSessionHistory` `0x72fcb0` |
| `DrTableCacheInfo::GetLogStr` | `0x62de90` | ` %.0f%%/%.0f%% %ums %u%%<br/>` (`0x18717e4`) |
| `VC::NCTalk::GetNCTalkErrorCodeToString` | `0x6043b0` | voice chat error code → text ❓ |
| `CLogServer`, `CFvLog`, `CManageLog2`, `CManageErrorCode` | `0x6fc8e0`–`0x703b30` | GameGuard code (out of scope) |

Built-in web hosts in the exe (all `<url>`; string table `0x14dfc8c`–`0x14dfe10`, `0x151febc`–`0x1520360`, `0x1579158`–`0x15792d0`, `0x166d414`–`0x166d8e0`, `0x17802b8`–`0x1780898`): publisher portals, payment, SNS, Tencent services. They are navigation allow-lists and defaults for the in-game browser ❓ (not traced one by one).

---

## 3. String anchors per subsystem

[`16-string-anchors.tsv`](16-string-anchors.tsv) has 3,478 rows. Rule for a row: the string is unique in the exe (C or UTF-16), it is 8 characters or more, it has `%`, `fail`, `error`, `invalid`, `cannot`, `timeout` or `disconnect`, and exactly one function uses it. Column `area` uses the chapter regexes of [00 §5](00-inventory.md) on the function name; `unnamed` = `sub_*`, `lib` = OpenSSL / zlib, `other` = named, no area match. URLs are `<url>`.

| Area | Rows | Area | Rows |
| --- | ---: | --- | ---: |
| 09-items | 781 | 12-infogate-arena | 99 |
| 11-social | 545 | 08-creature | 73 |
| unnamed | 493 | lib | 49 |
| 14-ui | 334 | 06-movement | 42 |
| 07-combat | 269 | 02-transport | 40 |
| 10-quests-npc | 262 | 01-startup | 7 |
| 16-strings-errors | 238 | 03-sts | 7 |
| 04-lobbygate | 126 | 13-datafile | 3 |
| other | 106 | 15-engine | 3 |
| | | 05-game-dispatch | 1 |

Best anchors (one search finds the function):

| Subsystem | String (381 address) | Function | 381 address |
| --- | --- | --- | --- |
| Quit / disconnect UI | `UI.QuitGame.NetworkErr` (`0x1634770`) | `QuitGameConfirmPanel::Open` | `0xa1a840` |
| Exit log | `end.log` (`0x16347b4`) | `QuitGameConfirmPanel::endProgress` | `0xa15f20` |
| Client log line | `%04d/%02d/%02d %02d:%02d:%02d.%03d, %s, %d, %s, %d, %s` (`0x151cce8`) | `ClientLog::write` | `0x10dc9d0` |
| Generic log file | `%sLog(%s)-%s.log` (`0x186c628`) | `sub_5D0F20` (LogFile init) | `0x5d0f20` |
| Library log file | `%s/%s_%I64d_%06d.log` (UTF-16, `0x186d3cc`, used at `0x5e66e4`) | `LogManager::CreateLogFile` | `0x5e6630` |
| Skill trace | `_skill.log` (`0x1650b30`) | `SkillTracer::Report` | `0x96bcf0` |
| Latency file | `..\log` (`0x14dd664`; `BattleRoyalField::initLog` also uses it) | `TaskSession::InitializeLatencyLogModule` | `0x876ed0` |
| Web log form fields | `ExecutionLog`, `UserLog` | `ClientExecutionLog::SendPostArguments`, `ClientUserLog::SendPostArguments` | `0x1167300`, `0x10e8df0` |
| WinINet errors | `HttpSendRequestW(POST) uri(%s), last-error(%u), time(%d)` (`0x159b870`); `HttpSendRequestEx(multi) …` (`0x159ad80`) | `HTTPClient::requestPost`; `HTTPClient::requestPostMultiPartsFormData` | `0xea1440`; `0xea19c0` |
| Crash | `EInterfaceErrorReport` (`0x14fe194`) | `LoadEngine` | `0x114c6d0` |
| Engine critical log | `[Engine] Critical Log: %s` (`0x14fdf04`) | `BNSClient::Tick` | `0x1167630` |
| Security exit | `exit by security. (%s)` (`0x14fdf38`) | `BNSClient::Tick` | `0x1167630` |
| STS / NP login (chat lib) | `[%p]:: ccLoginResult called.` (`0x186caf8`) | `_ccLoginResult` | `0x5ddbc0` |
| MAC address | `%0.2X-%0.2X-%0.2X-%0.2X-%0.2X-%0.2X` (`0x1576398`) | `GameSession::MakeMacAddress` | `0xefba60` |
| Web session log | `%02d-%02d:%02d:%02d,%c,%d,%u,%s` (`0x16afda0`) | `WebSessionManager::SessionLog::ToString` | `0x723750` |
| JSON errors | `Line %d, Column %d` (`0x179d674`) | `Json::OurReader::getLocationLineAndColumn` | `0x708d80` |
| Rule config option | `gamelog-url` (`0x178e7e0`) | table entry `0x19e1f80` | — |
| Game error names | table `0x194fc30` | `L_ERROR_CODE_TO_STRING` | `0x712bc0` |

---

## 4. Worked example

`ClientLog::Flush` sends one queued line. Example line `abc` (real lines are the full §2.3 format). `T_STRING` = `i16 n` + `n` UTF-16LE units ([05](05-game-dispatch.md)). Plain header = `size` i16 + `command` i16 ([02 §2.2](02-transport.md#22-plain-packet-header)).

| Off | Bytes | Field |
| ---: | --- | --- |
| 0 | `0C 00` | size = 12 |
| 2 | `97 01` | command 407 `G_WriteClientLogReqCmd` |
| 4 | `03 00` | `log` length = 3 |
| 6 | `61 00 62 00 63 00` | `"abc"` |

The session crypto then wraps this (02 §2.1). ❓ We did not capture a real 407 packet.

Quit example: GameD sends a kick with `DisconnectGameReason` 8. `GameSession::OnError` (`0xf44216`) → `sub_1129A90(…, 17)` → `QuitGameConfirmPanel::Open` → `UI.ExitGame.Kick.NoRespopnseServerPing.Title` / `.Confirm` → at exit `end.log` line `2020-5-1 12:0:0, 17, <+0x70>, <+0x74>`.

---

## 5. Open items ❓

1. The code that sets `ClientLog +0x10` (upload on). We think `G_RequestClientLogNotifyCmd` 410 (site `0xef64d0`; IDA has no function at this address).
2. The `Session::OnError(kind, a2, a3)` values from `Network` (§1.4): kind 2 / 3 and the meaning of a2 and a3.
3. Where the GameD `DisconnectGameReason` (`GameSession +0x60`) is written. No `mov [reg+60h], imm` in `GameSession` methods; the writer takes the value from a register or a packet.
4. Panel `+0x70` / `+0x74` sources (network reason, system code) in `QuitGameConfirmPanel::Open`.
5. Reader of `send-network-latency-to-web` (`+0x309`).
6. Crash report and dump: inside `bsengine_Shipping.dll` (`EInterfaceErrorReport`), not read.
7. `USERLOG_TYPE` per `FormattedWrite` call site, and the XML element names of `MakeLogXmlFile` (`0x11294e0`) and `MakeLogXmlString` (`0x10aac80`).
8. `frostStatic.log` owner (`sub_633590`).
9. Per-feature error-code → `Msg.*` alias maps (§1.2) are not listed one by one; use [`16-string-anchors.tsv`](16-string-anchors.tsv) and the feature chapters.
