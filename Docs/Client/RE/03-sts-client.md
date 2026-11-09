# 03 — Client.exe 381: STS client (login to StServers)

Summary:

1. The game talks to the STS gate (CliGate) through `NPSession`, network slot 2 (`NEXTPLATFORM`, see [02-transport §1.1](02-transport.md)). `NPSession` does not build the text messages itself. It loads `./StsMsgCli32.dll` (`NPStsMsgLib::Create` `0xbc90a0`) and calls the `StsMsgLib::IStsMsg` interface to build, encrypt, receive and parse STS messages.
2. Each request is a stack struct `StsMsgLib::_StsMsgRequest*` (419 GM types) with an `ECommand` id (for example 3101 = `Auth/LoginStart`) and a transaction id from the global counter `0x1b2f440`. The DLL writes it into an `NPRequestPacket` (64 148 bytes). The network layer sends the bytes as they are.
3. Login chain (password path): `Sts/Connect` (1001) → `Auth/LoginStart` (3101) → `Auth/KeyData` (3102) → `Auth/LoginFinish` (3103) → `GameAccount/ListMyAccounts` (3106) + `Auth/GetMyUserInfo` (3114) + `Auth/RequestToken` (3110). Each step is sent from the success handler of the step before it (§3).
4. Replies go through one dispatcher (`0xc25140`). A reply is a success only when the status is `200` and the reason is `OK`. Failures of the login steps go to `NPSession::ReceiveLoginFailure` (`0xc21c10`). It maps the error name or code to a `LOBBY_EVENT_LOGIN_FAILURE` reason for the UI (§5).
5. After login, the client keeps the NP link for ping (every 30 s), presence, custom messages and tokens. When `_npReconnectEnabled` is set, a lost link starts a reconnect state machine (`NPSession::Tick` `0xc24c50`, §6) that sends `Auth/RequestToken` on a timer and logs `[np-reconnect] …` lines.

```
 TitleLoginPanel::loginReq 0x7ff420 ─► Lobby::RequestLogin 0xce5910 ─► 0xbdd570 ─► NPSession::SendLoginRequest 0xbd40c0
                                                                                          │ connect slot 2
                                                                                          ▼
 NPSession::OnConnect 0xc22150 ─► IStsMsg::Initialize(programId, gameCode)
     │                            SendInitialConnectMessage 0xc08300  (1001 Sts/Connect)
     │                            SendLoginStartMessage     0xc076d0  (3101)  or SendWebLoginTokenStart 0xc08200 (3111)
     ▼
 Send: _StsMsgRequest* (stack) ─► 0xbce0f0 IStsMsg::Build ─► NPRequestPacket ─► RecordPacket 0xbdd680 ─► 0x47d530 send
 Recv: Session::IsCompleted 0xbce1e0 (IStsMsg::WriteReceiveData) ─► ToPacket 0xbd41d0 (IStsMsg::Parse)
       ─► NPResponsePacket ─► OnRecv 0xc253d0 ─► request 0xc08430 / response 0xc25140 / notify 0xbd4320

 3101 OK ─► 3102 OK ─► 3103 OK ─► ReceiveLoginFinishSuccess 0xc24d80
                                   ├─► [GReleaseConfig+0xA8 set] 3110 (type 1)
                                   ├─► 3106 ─► OK 0xc16990 ─► 3107
                                   ├─► 3114
                                   ├─► 3110 (type 3)
                                   └─► BNSClient::OnLogin
```

Sources: all facts come from the 381 database `ida/lanes/cx/Client.exe.i64`. "419 GM" marks a fact (name, type, enum) that comes only from `Client_gm.exe.i64`. ❓ marks a fact that we did not verify. The wire text (request line, XML, SRP, RC4) is made inside `StsMsgCli32.dll`. That DLL is described in [`../../Platform/Sts/07-client-login-wire.md`](../../Platform/Sts/07-client-login-wire.md) and [`../../Platform/Sts/02-messages.md`](../../Platform/Sts/02-messages.md). This chapter does not repeat it.

Many 381 PDB names near this code are false matches. The decompiler shows `Log::Write` at each sender, but that call returns the allocator. `SoulBoostProperty` ctor `0x5d3550` is the local IP getter. `EInterfaceSetClientInfo+1` is the `BNSClient*` (object at `0x1a4c0f8`). Use raw offsets, not the names.

Files next to this chapter:

| File | Content |
| --- | --- |
| [`03-sts-functions.tsv`](03-sts-functions.tsv) | 74 `NPSession` / `NPStsMsgLib` / NP packet functions with 381 address and role |
| [`03-sts-commands.tsv`](03-sts-commands.tsv) | Every `ECommand` that 381 sends or handles: id, name, direction, sender, reply handler |
| [`03-sts-enums-gm.tsv`](03-sts-enums-gm.tsv) | All 29 `StsMsgLib::*` enums of 419 GM (375 rows), then the lobby and NP enums of this chapter (37 rows, partial) |
| [`03-sts-strings.tsv`](03-sts-strings.tsv) | 40 strings of the NP code with 381 address, xref and function |

---

## Quick find

| Task | Function | 381 address | GM name | Section |
| --- | --- | --- | --- | --- |
| Load the STS DLL | `NPStsMsgLib::Create` | `0xbc90a0` | same | §1 |
| Next transaction id | `NPStsMsgLib::GetTransactionId` / global | `0xbc9050` / `0x1b2f440` | same | §2.2 |
| Build a request | `sub_BCE0F0` | `0xbce0f0` | ❓ | §2.2 |
| Remember txn → command | `NPSession::RecordPacket` | `0xbdd680` | same | §2.2 |
| Receive bytes | NPSession vt18 `IsCompleted` | `0xbce1e0` | `Session::IsCompleted` | §2.3 |
| Parse a message | NPSession vt19 `ToPacket` | `0xbd41d0` | `NPSession::ToPacket` | §2.3 |
| Route by message type | `NPSession::OnRecv` | `0xc253d0` | same | §2.3 |
| Route a reply | `sub_C25140` | `0xc25140` | `NPSession::ProcessResponse` ❓ | §2.4 |
| Start the login | `NPSession::SendLoginRequest` | `0xbd40c0` | same | §3.1 |
| Start STS after connect | `NPSession::OnConnect` | `0xc22150` | same | §3.1 |
| Send LoginStart | `NPSession::SendLoginStartMessage` | `0xc076d0` | same | §3.2 |
| Send KeyData / LoginFinish | `sub_C074B0` / `sub_C073A0` | `0xc074b0` / `0xc073a0` | `SendLoginKeyData` / `SendLoginFinish` ❓ | §3.2 |
| Login success | `NPSession::ReceiveLoginFinishSuccess` | `0xc24d80` | same | §3.3 |
| Login failure → UI | `NPSession::ReceiveLoginFailure` | `0xc21c10` | same | §5 |
| Request a token | `NPSession::SendTokenRequest` | `0xc24710` | same | §3.4 |
| Token reply | `ReceiveTokenSuccess` / `ReceiveTokenFailure` | `0xc24340` / `0xc24130` | same | §3.4 |
| Game token | `sub_C063C0` | `0xc063c0` | `SendRequestGameToken` ❓ | §3.5 |
| Ping | NPSession vt12 → `sub_C067E0` | `0x4e25a0` → `0xc067e0` | ❓ | §7.1 |
| Server kick (3201) | `sub_C08430` → `0xc06250` | `0xc08430` | ❓ | §7.2 |
| Logout | `NPSession::SendLogoutRequest` | `0xc075b0` | same | §7.3 |
| Link error | `NPSession::OnError` | `0xc223a0` | same | §6 |
| Reconnect timers | `NPSession::Tick` | `0xc24c50` | same | §6 |

Search anchors: wide string `./StsMsgCli32.dll` `0x15f2674`; ASCII `CreateStsMsg` `0x15f2698`; wide `[np-reconnect]` strings `0x15f26c8`–`0x15f2d18`; `ErrBadPasswd` `0x15f2c60`; the NPSession vtable `0x1604cd0`; the source path `…\ClientEXE\Private\NPSession.cpp` `0x15f2760`. IDA's string list does not show these wide strings. Scan `0x15f2600`–`0x15f2f90`.

---

## 1. Objects

### 1.1 NPSession vtable (`0x1604cd0`, 23 slots)

| Slot | 381 address | Role |
| --- | --- | --- |
| 0 | `0x4e2af0` | dtor |
| 1 | `0x4031e0` | `Id`: returns 2 |
| 2 | `0x4e2590` | `Version`: returns 122 (DLL version 0.122) |
| 3 | `0x403a90` | `CryptAlgorithm`: returns 0 (no game crypt; the DLL encrypts) |
| 4 | `0xc223a0` | `OnError` (§6) |
| 6 | `0xc22150` | `OnConnect` (§3.1) |
| 8 | `0xc43e80` | `OnClose`: logs line 324 `[np-reconnect] OnClose reason:%d` (`0x15f2a18`), sets reconnect step 1. The rest is not read ❓ |
| 10 | `0xc253d0` | `OnRecv` (§2.3) |
| 12 | `0x4e25a0` | `SendPing`: calls `0xc067e0` (§7.1) |
| 17 | `0xbc8e80` | `PacketSize`: returns the size as it is |
| 18 | `0xbce1e0` | `IsCompleted` (§2.3) |
| 19 | `0xbd41d0` | `ToPacket` (§2.3) |
| 20 | `0xbce170` | `ToStream`: copies `NPRequestPacket._buffer` for `Size()` bytes; returns −size if the target is too small |

### 1.2 NPSession fields (381 offsets)

381 offset = 419 GM offset − 32 for the fields after `+0x2c`.

| Offset | Field (GM name) | Set / used by |
| --- | --- | --- |
| `+0x2c` | `_programId` | `OnConnect` → `IStsMsg::Initialize` |
| `+0x30` | `_npGameCode` (wstring) | `OnConnect` |
| `+0x4c` / `+0x50` | account / password (`wchar_t*`) | `SendLoginStartMessage`; password freed after use |
| `+0x58` | local IP (char[55]) | `SendInitialConnectMessage` |
| `+0xA4` / `+0xC4` / `+0xC8` | clientAppGuid / clientAppGroupId / serverAppGuid | `OnConnect` (`UuidFromStringW` of globals `0x1a82248`, `0x1a82260`, `0x1a8222c`) |
| `+0xDC` | game alias (wstring) ❓ | `sub_C063C0` (3107) |
| `+0x160` | `_languageType` | `sub_C073A0` (3103) |
| `+0x164` | `NPStsMsgLib` (+0 create, +4 cleanup, +8 HMODULE, +12 `IStsMsg*` = `+0x170`, +16 init = `+0x174`) | all |
| `+0x178` | `_disconnectLobbyReason` | `0xc06250`, `OnError` |
| `+0x17C` | `_secondaryAuthType` | `OnConnect`, `ReceiveLoginFailure`, `ReceiveLoginFinishSuccess` |
| `+0x19C` / `+0x1A0` | reconnect retry count / `ReconnectionStep` | §6 |
| `+0x1A8` / `+0x1B0` | reconnect timer / limit | §6 |
| `+0x1B8` | `_transactionToTokenMap` (txn → `WebSessionTokenType`) | `SendTokenRequest`, `ReceiveToken*` |
| `+0x1C8` / `+0x1D0` | request-token timer / limit | §6 |
| `+0x1D8` | `_receivedLogoutRequest` | `0xc06250`, `OnError` |

### 1.3 StsMsgCli32.dll and IStsMsg

`NPStsMsgLib::Create` `0xbc90a0` frees the old lib, calls `LoadLibraryW("./StsMsgCli32.dll")` (`0x15f2674`), gets `CreateStsMsg` (`0x15f2698`) and `CleanUpStsMsg` (`0x15f26a8`), and calls create. `NPStsMsgLib::Cleanup` `0xbc9060` only clears the init flag.

| `IStsMsg` slot (419 GM) | Method | 381 user |
| --- | --- | --- |
| +0 | `Initialize(int programId, const char* gameCode)` | `OnConnect`; result 1 sets `+0x174` = 1 |
| +4 | `Build(_StsMsg*, _StsMsgBuffer*)` | `0xbce0f0` |
| +8 | `WriteReceiveData(u8*, u32)` | `0xbce1e0` |
| +12 | `Parse(_StsMsg**, callback)` | `0xbd41d0` with callback `0xbd3f20` |
| +16 | `ReleaseStsMsg` | ❓ |
| +20 | `SetCryptFilterWhenUsingExternalAuth` | `sub_C16B10` (3112 OK) |

---

## 2. Message format

### 2.1 Structs (419 GM)

| Struct | Size | Fields (offset name) |
| --- | --- | --- |
| `_StsMsgRequest` | 16 + body | 0 vtable, 4 `eCmd`, 8 `msgType` (1 = REQUEST), 12 `nTxnId`, 16+ body |
| `_StsMsgResponse` | 80 + body | 0 vtable, 8 `eCmd`, 12 `msgType`, 16 `nTxnId`, 20 `bComplete`, 24 `pszErr`, 28 `nStatusCode`, 32 `pwszReason`, 36 `nErrorCode`, 40 `nBanId`, 48 `tEffectiveFrom`, 56 `tEffectiveTo`, 64 `tUserUnbanFrom`, 72 `nBanReasonCode`, 80+ body |
| `NPRequestPacket` | `0xFA94` | 0 vt, 4 `{pData, nLen}`, 12 `_buffer[64000]`, `0xFA0C` command, `0xFA10` txn, `0xFA14` customCommand[128] |
| `NPResponsePacket` | `0x8C` | 0 vt, 4 `_stsmsg`, 8 `_stsmsglib`, 12 customCommand[128] |

`EMessageType` (419 GM): 1 REQUEST, 2 RESPONSE, 3 NOTIFY.

### 2.2 Send path

All senders use the same steps (for example `0xc076d0`, `0xc24710`):

1. Stop if the init flag `+0x174` is 0.
2. Fill the request struct on the stack: command, `msgType` = 1, txn = `++dword_1B2F440`. The 381 vtable has the false name `_StsMsgRequestGoodsListCategoryIds`.
3. Get the allocator, then `_aligned_malloc(0xFA94, 16)`.
4. `NPRequestPacket::NPRequestPacket(txn, cmd)` `0xbce2e0`.
5. `0xbce0f0` calls `IStsMsg::Build`. It needs result 1 and `nLen` > 0.
6. `NPSession::RecordPacket` `0xbdd680` puts txn → {cmd, customCommand} into the static map `_msglist` `0x1a16c70`.
7. `sub_47D530` sends the packet.

`SendInitialConnectMessage` `0xc08300` uses txn 0 and does not call `RecordPacket`. Its body is `pszIpAddress` = `inet_ntoa` of the first local IPv4 address (`0x5d3550`: `gethostname` + `gethostbyname`). The text is also kept at `+0x58`.

### 2.3 Receive path

1. vt18 `0xbce1e0` gives the bytes to `IStsMsg::WriteReceiveData`. It returns 1 only when the lib is ready.
2. vt19 `0xbd41d0` calls `IStsMsg::Parse` with callback `0xbd3f20` (txn → command from `_msglist`). It makes an `NPResponsePacket` and copies customCommand from the map.
3. `OnRecv` `0xc253d0` switches on `GetMessageType`: 1 → `0xc08430` (server request), 2 → `0xc25140` (reply), 3 → `0xbd4320` (notify). Then it gives the packet (vt+4) to the World (`BNSClient+0x4c`), Lobby (`+0x48`) and Ui (`+0x44`). Then `0xbd3e50` removes the txn from `_msglist`.

### 2.4 Reply dispatcher `0xc25140`

OK = `nStatusCode` (+28) == 200 **and** `wcscmp(pwszReason, "OK")` == 0. Lobby events go to `BNSClient+0x48` (`Lobby::ProcessLobbyEvent2`, vt+4).

| Cmd | OK | Fail |
| --- | --- | --- |
| 3101 | `0xc16af0` → send 3102 | `0xc21c10` |
| 3102 | `0xc16ad0` → send 3103 | `0xc21c10` |
| 3103 | `0xc24d80` | `0xc21c10` |
| 3105 | `0xbdd2b0` → `0xbd3fb0` | `0xbce0d0` |
| 3106 | `0xc16990`: copy the first account alias to `+0xDC` ❓, then send 3107 (`0xc16a20`) | `0xbce0b0` |
| 3107 | `0xbf2c70` (if field +40 == 1 → `0xbf2bf0`) | `0xbce030` (field +40 = 0) |
| 3110 | `0xc24340` | `0xc24130` |
| 3111 | `0xc16b40` → send 3112 | `0xc21c10` |
| 3112 | `0xc16b10`: `IStsMsg` +20, then send 3103 | `0xc21c10` |
| 3113 | `0xc16a80`: send 3103, lobby event 4 | `0xbdd440` |
| 3114 | `0xbcdf70` | — |
| 3116 | `0xc16a30`: send 3103, lobby event 4 | `0xbdd2d0` |
| 3117 | lobby event 3 (`0xbd3e10`) | lobby event 3 (`0xbd3dd0`) |
| 1501 | `0xc1df90` | `0xbce0b0` |
| 1601 | `0xbdcf10` | `0xbdceb0` |

There is no case for 1002 (ping), 3118, 3202 and 3501 ❓ (no reply action found).

---

## 3. Login

### 3.1 Connect and first message

`NPSession::OnConnect` `0xc22150`:

1. Convert `_npGameCode` to MBCS. Call `IStsMsg::Initialize(+0x2c, gameCode)`. Result 1 → `+0x174` = 1.
2. If reconnect step == 2: send 1001, then 3111 (§6).
3. Else: `+0x17C` = 0. Fill the app GUIDs (§1.2).
4. If the web flag (byte at `0x1a4c0f7` ❓, see [`Sts/03-login-flow.md`](../../Platform/Sts/03-login-flow.md)) is set: send 1001 + 3111. A send failure → disconnect reason 102.
5. Else, if programId ≠ 0 and gameCode ≠ empty: send 1001 + 3101. Else disconnect reason 105 (`0x5d26e0`).

### 3.2 Request fields

| Cmd | Sender | Body (419 GM `_StsMsgRequest*`) | Value in 381 |
| --- | --- | --- | --- |
| 1001 | `0xc08300` | `pszIpAddress` @16 | local IPv4 text |
| 3101 | `0xc076d0` | `pszLoginName` @16, `pszPassWord` @20 | account `+0x4c` and password `+0x50`, each `WideCharToMultiByte(CP_ACP)` into a 64-byte buffer. Then the wide password is zero-filled, freed (`0x4015d0`) and `+0x50` = 0 |
| 3102 | `0xc074b0` | none | — |
| 3103 | `0xc073a0` | `eLanguageType` @16 | `+0x160` |
| 3111 | `0xc08200` | none | — |
| 3112 | `0xc07890` | token bytes, Base64 flag, provider code, AppId | text from `makeInnovaAuthnTokenStr` `0xbc8da0` ❓ (provider `tencent` `0x15f2a94`) |

The SRP math and the RC4 switch after 3102 are inside the DLL ([07-client-login-wire](../../Platform/Sts/07-client-login-wire.md)).

### 3.3 LoginFinish success `0xc24d80`

1. If `_npReconnectEnabled` (`GRuleConfig+0x5CC`) and step == 2: log line 2725 `[np-reconnect] reconnected. retry count:%d` (`0x15f2b88`), step = 0, retry = 0, `SendTokenRequest(3)`. End.
2. Else, switch on the auth type of the reply. Case 2 stores it in `+0x17C` and can open the OTP panel (`UI.LoginOtp.OtpExpired.Reserve` `0x15f2be0`, `ClientConfig+0x5520`) ❓.
3. Loop over the role ids. If one role == 8, set a flag in the Lobby (`+0x68`) ❓.
4. If `GReleaseConfig+0xA8` is a non-empty text: `SendTokenRequest(1)`.
5. Send 3106 (`0xc06d60`), 3114 (`0xc072a0`), `SendTokenRequest(3)`, then `BNSClient::OnLogin`.

This order agrees with [07-client-login-wire](../../Platform/Sts/07-client-login-wire.md) (3106 → 3114 → 3110).

### 3.4 RequestToken (3110)

`NPSession::SendTokenRequest(type)` `0xc24710`:

- Guard: init flag set and reconnect step == 0.
- `Token.appId` @16 by type: 1 INTERNAL → wstring `0x1a82268`; 2 EXTERNAL → `0x1a82284`; 3 NP_TOKEN_FOR_RECONNECT → `0x1a82248`; other → no send.
- After the send: `_transactionToTokenMap[txn] = type` (`+0x1B8`). For type 1: `WebSessionManager+4` = 2 (under its lock). If the send failed: `WebSessionManager::StartRetryLoginTimer(false)`.

`ReceiveTokenSuccess` `0xc24340` takes the type from the map and erases the entry. For 3110 it copies `authnToken` (+80) into the global wstring `0x1a4f3a4` ❓ (a `WebSessionManager` token). For a web type ❓ it makes a URL from `InGameWeb::GetDomainUrl` + `/login/sso?` or `/game/sso?` + `app_id`, `&target_app_id`, `&authn_token=`, `&return_url=` (`0x15f2e6c`–`0x15f2ee8`) and opens it with `ShellExecuteW`. An empty token shows `UI.Confirm.FailedToCharge.NullAuthnToken` (`0x15f2f08`).

`ReceiveTokenFailure` `0xc24130` erases the map entry and opens a confirm panel with `UI.Confirm.FailedToCharge.FailedToReceiveToken` (`0x15f2f60`).

### 3.5 Game token (3107) and LobbyGate

`sub_C063C0` `0xc063c0` sends 3107 with `pwszGameAlias` = `+0xDC`, only when that text is not empty. It is called from the 3106 success handler `0xc16990`. ❓ How the `guidGameToken` reply goes to LobbyGate (`BNSClient::EnterGameForNPWithLobbyGate` `0x1157fbd`, GM `G_NPTokenAuthReqPacket`, `NPMsgState`) is not read.

### 3.6 Differences with the StServers docs

| Topic | Client 381 | [11-cligate-protocol](../../Platform/StServers/11-cligate-protocol.md) / [15-authsrv](../../Platform/StServers/15-authsrv.md) |
| --- | --- | --- |
| Command ids | Library `ECommand` 3101/3102/3103 … (ABI only, not on the wire) | Gate and AuthSrv route by the text name (`Auth/LoginStart`); AuthSrv uses its own internal ids |
| Ping | 1002 every 30 s (vt12); no reply case | ❓ Server reply rule not compared; the client ignores any reply |
| After LoginFinish | 3106, 3114, 3110 at once; 3107 after 3106 | Server must accept them in any order ❓ |
| Error text | Client checks reason names `ErrAccountNotFound`, `ErrBadParam`, `ErrBadPasswd`, `ErrDoubleLogin`, `ErrAccsNotAllowedIp` and numeric codes (§5) | Server docs must send the same names and codes ❓ |
| Server request | Client accepts 3201 (`Presence/LogoutUser` kick) as a request and replies after 3.0 s | ❓ not in the server docs |
| Notify | Client handles 1201 and 3761 | ❓ |

---

## 4. Worked example — password login

User types `alice` (no separator, so `Lobby::RequestLogin` adds `_npAccountPostfix`). Password `keyhere`.

1. `OnConnect` → `Initialize(programId, gameCode)` → `+0x174` = 1.
2. 1001: txn 0, `pszIpAddress` = `"192.168.1.10"`.
3. 3101: txn = `0x1b2f440` + 1 (for example 1). `pszLoginName` = `"alice<postfix>"`, `pszPassWord` = `"keyhere"` (MBCS, 64-byte buffers). The wide password at `+0x50` is erased. `_msglist[1]` = 3101.
4. Reply type 2, cmd 3101, status 200, reason `OK` → `0xc16af0` → 3102 (txn 2).
5. 3102 OK → 3103 (txn 3, `eLanguageType` = `+0x160`).
6. 3103 OK → 3106 (txn 4), 3114 (txn 5), 3110 (txn 6, type 3), `BNSClient::OnLogin`.
7. If step 3 returns status ≠ 200 with reason `ErrBadPasswd`: `ReceiveLoginFailure` posts event 7 with `_reason` = 10 ❓ (UI text not mapped).

---

## 5. Errors shown to the user

`NPSession::ReceiveLoginFailure` `0xc21c10` calls `printResponse` `0xbc8e20`. It posts `LOBBY_EVENT_LOGIN_FAILURE` (7) with `_LobbyFailEvent{_reason i16 @4, _systemReason @8}`. Default `_reason` = 1, `_systemReason` = 0.

| Cmd | Condition | `_reason` / action |
| --- | --- | --- |
| 3101 | reason `ErrAccountNotFound` (`0x15f2c20`) | 8 |
| 3101 | reason `ErrBadParam` (`0x15f2c48`) | 9 |
| 3102 | reason `ErrBadPasswd` (`0x15f2c60`) | 10 |
| 3102 | errorCode 3344 | 11 |
| 3103 | reason `ErrDoubleLogin` (`0x15f2c7c`) | 12 |
| 3103 | reason `ErrAccsNotAllowedIp` (`0x15f2c9c`) | default (1) |
| 3103 | errorCode 3745 / 4100 / 3015 / 3352 / 3766 / 3717 / other | 19 / 20 / 21 / 13 / 22 / 14 / 18 |
| 3103 | errorCode 3361 and auth type (+100) == 2 | `+0x17C` = 2; `BUiLobby::OpenOTPConfirmPanel` (`UI.LoginOtp.OtpExpired.ExitGame.Confirm` `0x15f2cc8`); no event 7 |
| 3103 | errorCode 3746 | event 8 `LOGIN_FAILURE_BANNED` with the ban times (+48, +56, +64) |
| 3111 | any | `_systemReason` = 107 (`LOGINFAIL_NP_ETC`) |
| 3112 | any | `_systemReason` = 100 (`LOGINFAIL_NP_AUTH`) |
| any | reconnect step == 2 | no UI. Log line 3046 (`0x15f2d18`), step = 1, restart timer |

Other failures: `0xbdd440` (3113) and `0xbdd2d0` (3116) post OTP events ❓. ❓ The text alias for each `_reason` (in `UiLobbyStateLogin::ProcessNPEvent` `0x7e3600`) is not read.

Quit panel `sub_1129A90(state)` `0x1129a90` sets `BNSClient+0x74` = state (`QuitGameConfirmPanel::State`, 419 GM): 8 CONNECTION_LOST, 12 DUPLICATE_LOGIN, 13 CREDENTIAL_CHANGED.

---

## 6. Link error and reconnect state machine

Config (`GRuleConfig`, 419 GM names): `+0x5CC` `_npReconnectEnabled`, `+0x5D0` `_npReconnectIntervalSeconds` (float s), `+0x5D4` `_npReconnectMaxRetryCount`, `+0x5D8` `_npReconnectRequestTokenIntervalSeconds` (float s). Values come from the rule config ❓ (not read).

`OnError` `0xc223a0`: `+0x174` = 0. If not `_receivedLogoutRequest`, reconnect enabled, `_disconnectLobbyReason` == 0 and the GameSession (`BNSClient+0x24`) is connected → log line 155 (`0x15f26c8`), step = 1, start the timer. Else `+0x17C` = 0 and by reason: 7 → quit panel 12; 8 → 13; 9 → 8; other → if game and World are present, log line 184 (`0x15f2848`) and `0x1157e30(…, 1)`. Then it cleans the `WebSessionManager` (`RetryLoginTimerEventListener::StopTimer`, `SetEvent(0x1a4d59c)`).

| Step (`+0x1A0`) | Event | Next | Action (`Tick` `0xc24c50` unless noted) |
| --- | --- | --- | --- |
| 0 INVALID | `OnError` (guards above) or `OnClose` | 1 | log line 155 or 324 |
| 1 DETECTED | timer > interval s, retry < max, GameSession connected, `SendCleanUserConnectionInfoInCligateReq` true | 2 | ++retry; reconnect through GameSession/CliGate ❓ |
| 1 DETECTED | timer > interval s, other case | 0 | quit panel 8 (CONNECTION_LOST) |
| 2 RECONNECTING | `OnConnect` | 2 | send 1001 + 3111; on failure log 225 / 232, disconnect 102, step 1 |
| 2 RECONNECTING | 3103 OK | 0 | log 2725, retry = 0, `SendTokenRequest(3)` |
| 2 RECONNECTING | login failure | 1 | log 3046, restart timer |
| 2 RECONNECTING | timer > 60 000 ms | 1 | — |

Token timer: when connected and reconnect is enabled, `Tick` calls `SendTokenRequest` each time `+0x1C8` passes `_npReconnectRequestTokenIntervalSeconds` × 1000 ms.

---

## 7. Calls after login

### 7.1 Ping (1002)

The keepalive `0x5d28f0` calls vt12 every 30 000 ms ([02-transport](02-transport.md)). NPSession vt12 `0x4e25a0` → `0xc067e0` sends 1002 with no body. The dispatcher has no 1002 case.

### 7.2 Server kick (3201 request)

`0xc08430` accepts only cmd 3201 and jumps to `0xc06250`:

1. `_receivedLogoutRequest` (`+0x1D8`) = 1.
2. If NP is connected: `0x4e9e20` registers a delayed call (`EVENT_TIME_OUT` `0x14b1668` = 3.0 s) to `0xc06110`, which sends the 3201 reply (msgType 2).
3. If the GameSession is connected: `GameSession+0x60` = 16 when the request field @16 == 4, else 1.
4. `_disconnectLobbyReason` by field @16: 2 → 7 DUPLICATE_LOGIN_KICKED_NP, 3 → 8 CREDENTIAL_CHANGED_KICKED_NP, 4 → 4 ❓, other → 9 SYSTEM_KICKED_NP. It is copied to `LobbyGateSession+0x8C` (`BNSClient+0x30`).

### 7.3 Logout (3105)

`NPSession::SendLogoutRequest` `0xc075b0` sends 3105 with no body. Callers: `LobbyGateSession::OnError` `0xca70bf` and `0xd0d7f8`. OK → `0xbdd2b0` → `0xbd3fb0` ❓ (not read).

### 7.4 Presence, slots and custom messages

| Cmd | Sender | Caller |
| --- | --- | --- |
| 3202 | `0xc064d0` (XML `<FriendName>…</FriendName><Msg>…</Msg>`) | `World::sendPresenceMessageWhisper` `0x740565` |
| 1501 | `0xc068c0` (`activity`), `0xc06a50` (`customize`), `0xc06be0` | `0xd0d720`, `0xd0d740`, `0xd0d760` |
| 1601 | `0xc05fe0` | `Friends::RequestAcceptProposal`, `NPAuction::RequestBid`, `Attendance::RequestAttendPromotion`, … |
| 3501 | `0xc26f90` | `SaveNpOption` `0x116a810`, `SaveNoLoadNpOption` `0x116a50a` |
| 3113 / 3116 / 3117 / 3118 | `0xc07190` / `0xc07080` / `0xc06e60` / `0xc06f70` | `Lobby::RequestVerifySecondaryAuth`, `LoginOtpPanel` |

Notify (`0xbd4320`): 1201 `COMMON_PRES_USER_INFO` → `0xbd3d10`; 3761 `CLI_FRIEND_PROPOSAL_ACCEPTED` → `0xbce060` (only when the Lobby exists).

---

## 8. Open items

- ❓ LobbyGate hand-off of the game token (§3.5).
- ❓ UI text for each `_reason` (§5).
- ❓ `OnClose` after the log line; logout success `0xbd3fb0`.
- ❓ Config values of the reconnect timers.
