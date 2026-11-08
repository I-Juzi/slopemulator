# 02 — Link command tables

✅ Target tables and handler pointers were read in 307. ❓ Names without an independent anchor remain proposed 348 identities.
Format literals below include the command code `h`. They are not body-only formats.
✅ Each listed literal has a direct reference from the target handler. ❓ Receive/send roles and field names are not assigned unless stated.
A handler can reference reply formats as well as its receive format. Do not use this list as a complete codec spec.

## Table index

| Link | 307 table | 348 table | 307 dispatcher | Valid command range | Target entries |
| --- | --- | --- | --- | --- | --- |
| Manage | ✅ `0x1401f9530` | ❓ `0x140202530` | ✅ `0x140059D60` | ✅ 1..16 | ✅ 6 |
| LobbyGate | ✅ `0x1401f9640` | ❓ `0x140202640` | ✅ `0x1400BC750` | ✅ 1..110 | ✅ 35 |
| InfoGate | ✅ `0x1401f9d30` | ❓ `0x140202d30` | ✅ `0x14002E490` | ✅ 1..10 | ✅ 3 |
| GameDaemon | ✅ `0x1401f9de0` | ❓ `0x140202de0` | ✅ `0x14007F920` | ✅ 1..109 | ✅ 51 |
| AccountInventory | ✅ `0x1401fa4c0` | ❓ `0x140203520` | ✅ `0x1400075C0` | ✅ 1..36 | ✅ 23 |

✅ Null handler entries fail dispatch. The target dispatchers use the command as a 16-byte array index.
❓ Guide dispatcher bodies were not all checked. Guide table names and function names came from PDB data.

## Commands

### Manage

| Command | 348 handler name | 307 handler / slot | Fragment flag | Referenced format literals in 307 |
| --- | --- | --- | --- | --- |
| ✅ 1 | ❓ `HandleLobbyManage_HelloReq`; 348 `0x1400587d0` | ✅ `0x1400572e0` / `0x1401f9540` | ✅ 0 | ✅ `hh` @ `0x14012e610`; ✅ `h` @ `0x14015aa94`; ✅ `hhh` @ `0x1401608f0` |
| ✅ 4 | ❓ `HandleLobbyManage_AnnounceNotify`; 348 `0x1400589b0` | ✅ `0x1400574c0` / `0x1401f9570` | ✅ 0 | ✅ `hs` @ `0x140160bac` |
| ✅ 5 | ❓ `HandleLobbyManage_KickReq`; 348 `0x140058b00` | ✅ `0x140057610` / `0x1401f9580` | ✅ 0 | ✅ `hqo` @ `0x14012c788`; ✅ `K` @ `0x14017afb0` |
| ✅ 8 | ❓ `HandleLobbyManage_CleanupGameAccountIdIndexReq`; 348 `0x1400590a0` | ✅ `0x140057bb0` / `0x1401f95b0` | ✅ 0 | ✅ `hqo` @ `0x14012c788` |
| ✅ 11 | ❓ `HandleLobbyManage_CleanupPcidIndexReq`; 348 `0x140059470` | ✅ `0x140057f80` / `0x1401f95e0` | ✅ 0 | ✅ `hqqh` @ `0x140161a18` |
| ✅ 14 | ❓ `HandleLobbyManage_RetryMileageDepositReq`; 348 `0x140059820` | ✅ `0x140058330` / `0x1401f9610` | ✅ 0 | ✅ `hqq` @ `0x140161e14` |

### LobbyGate

| Command | 348 handler name | 307 handler / slot | Fragment flag | Referenced format literals in 307 |
| --- | --- | --- | --- | --- |
| ✅ 1 | ✅ `HandleLobbyLobbyGate_HelloReq`; 348 `0x1400c25c0` | ✅ `0x1400bdc90` / `0x1401f9650` | ✅ 0 | ✅ `hhhh` @ `0x14012c4ec`; ✅ `hh` @ `0x14012e610`; ✅ `hiChhsQQiihhiQQQQQQQ;QQqCqqiQiic;` @ `0x1401bc0d8` |
| ✅ 6 | ✅ `HandleLobbyLobbyGate_InitSessionWithNPTokenReq`; 348 `0x1400c2d90` | ✅ `0x1400be4c0` / `0x1401f96a0` | ✅ 0 | ✅ `hqooi` @ `0x1401bc758` |
| ✅ 9 | ✅ `HandleLobbyLobbyGate_StartSessionWithNPTokenReq`; 348 `0x1400c3380` | ✅ `0x1400beab0` / `0x1401f96d0` | ✅ 0 | ✅ `hqoo` @ `0x1401bccec` |
| ✅ 12 | ✅ `HandleLobbyLobbyGate_InitSessionWithLobbyTokenReq`; 348 `0x1400c3890` | ✅ `0x1400befc0` / `0x1401f9700` | ✅ 0 | ✅ `hqqqi` @ `0x14017d720` |
| ✅ 15 | ✅ `HandleLobbyLobbyGate_StartSessionWithLobbyTokenReq`; 348 `0x1400c3e70` | ✅ `0x1400bf5a0` / `0x1401f9730` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 18 | ✅ `HandleLobbyLobbyGate_LostAnonymousConnectionNotify`; 348 `0x1400c4180` | ✅ `0x1400bf8b0` / `0x1401f9760` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 19 | ✅ `HandleLobbyLobbyGate_LostConnectionNotify`; 348 `0x1400c43f0` | ✅ `0x1400bfb20` / `0x1401f9770` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 23 | ✅ `HandleLobbyLobbyGate_LogoutNotify`; 348 `0x1400c4650` | ✅ `0x1400bfd80` / `0x1401f97b0` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 24 | ✅ `HandleLobbyLobbyGate_GetEnterWorldTokenReq`; 348 `0x1400c48b0` | ✅ `0x1400bffe0` / `0x1401f97c0` | ✅ 0 | ✅ `hqo` @ `0x14012c788` |
| ✅ 28 | ✅ `HandleLobbyLobbyGate_GetWaitInfoReq`; 348 `0x1400c4df0` | ✅ `0x1400c0520` / `0x1401f9800` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 31 | ✅ `HandleLobbyLobbyGate_GiveupWaitReq`; 348 `0x1400c5040` | ✅ `0x1400c0770` / `0x1401f9830` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 35 | ✅ `HandleLobbyLobbyGate_GetCharacterInfoListReq`; 348 `0x1400c5360` | ✅ `0x1400c0a90` / `0x1401f9870` | ✅ 0 | ✅ `hq` @ `0x140130828`; ✅ `hiqqhCqohsbqQQi;` @ `0x1401bf528` |
| ✅ 38 | ✅ `HandleLobbyLobbyGate_GetCharacterInfoReq`; 348 `0x1400c5630` | ✅ `0x1400c0dd0` / `0x1401f98a0` | ✅ 0 | ✅ `hqo` @ `0x14012c788`; ✅ `hqqohsbqQQi` @ `0x1401bf8d8` |
| ✅ 41 | ✅ `HandleLobbyLobbyGate_CheckCharacterNameReq`; 348 `0x1400c5ae0` | ✅ `0x1400c1270` / `0x1401f98d0` | ✅ 0 | ❓ Not found |
| ✅ 44 | ✅ `HandleLobbyLobbyGate_CreateCharacterReq`; 348 `0x1400c5b10` | ✅ `0x1400c12a0` / `0x1401f9900` | ✅ 0 | ✅ `hqohsbQ` @ `0x1401bfb88` |
| ✅ 47 | ✅ `HandleLobbyLobbyGate_DeleteCharacterReq`; 348 `0x1400c6190` | ✅ `0x1400c1920` / `0x1401f9930` | ✅ 0 | ✅ `hqo` @ `0x14012c788`; ✅ `hqoQE` @ `0x1401b9f70` |
| ✅ 51 | ✅ `HandleLobbyLobbyGate_CancelWaitCharacterDeletionReq`; 348 `0x1400c67e0` | ✅ `0x1400c1f60` / `0x1401f9970` | ✅ 0 | ✅ `hqo` @ `0x14012c788`; ✅ `hqoQE` @ `0x1401b9f70` |
| ✅ 54 | ✅ `HandleLobbyLobbyGate_RecoverCharacterReq`; 348 `0x1400c6e30` | ✅ `0x1400c25a0` / `0x1401f99a0` | ✅ 0 | ✅ `hqo` @ `0x14012c788`; ✅ `hqoQE` @ `0x1401b9f70` |
| ✅ 57 | ✅ `HandleLobbyLobbyGate_BindCharacterSlotReq`; 348 `0x1400c7540` | ✅ `0x1400c2ca0` / `0x1401f99d0` | ✅ 0 | ✅ `hqoq` @ `0x14012cbd4` |
| ✅ 60 | ✅ `HandleLobbyLobbyGate_GetChargeInfoReq`; 348 `0x1400c7af0` | ✅ `0x1400c3250` / `0x1401f9a00` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 64 | ✅ `HandleLobbyLobbyGate_GetGoodsCountReq`; 348 `0x1400c7df0` | ✅ `0x1400c3550` / `0x1401f9a40` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 67 | ✅ `HandleLobbyLobbyGate_ListGoodsReq`; 348 `0x1400c8080` | ✅ `0x1400c37e0` / `0x1401f9a70` | ✅ 0 | ✅ `hqc` @ `0x1401c1cfc` |
| ✅ 70 | ✅ `HandleLobbyLobbyGate_TakeGoodsReq`; 348 `0x1400c83d0` | ✅ `0x1400c3b30` / `0x1401f9aa0` | ✅ 0 | ✅ `hqq` @ `0x140161e14` |
| ✅ 73 | ✅ `HandleLobbyLobbyGate_ListLobbyServiceItemInfoReq`; 348 `0x1400c85e0` | ✅ `0x1400c3d40` / `0x1401f9ad0` | ✅ 0 | ✅ `hqQq` @ `0x1401b9f9c` |
| ✅ 76 | ✅ `HandleLobbyLobbyGate_GetLobbyServiceItemInfoReq`; 348 `0x1400c8860` | ✅ `0x1400c3fc0` / `0x1401f9b00` | ✅ 0 | ✅ `hqq` @ `0x140161e14` |
| ✅ 79 | ✅ `HandleLobbyLobbyGate_GetServiceEntryListReq`; 348 `0x1400c8aa0` | ✅ `0x1400c4200` / `0x1401f9b30` | ✅ 0 | ✅ `hq` @ `0x140130828`; ✅ `hiqCqQqohq;` @ `0x1401c2b08` |
| ✅ 82 | ✅ `HandleLobbyLobbyGate_ChangeCharacterAppearanceWithAccountInventoryItemReq`; 348 `0x1400c8d00` | ✅ `0x1400c44c0` / `0x1401f9b60` | ✅ 0 | ✅ `hqobqqi` @ `0x1401c2c78` |
| ✅ 85 | ✅ `HandleLobbyLobbyGate_ChangeCharacterSexWithAccountInventoryItemReq`; 348 `0x1400c9300` | ✅ `0x1400c4ac0` / `0x1401f9b90` | ✅ 0 | ✅ `hqocbqqi` @ `0x1401c3450` |
| ✅ 88 | ✅ `HandleLobbyLobbyGate_ChangeCharacterRaceWithAccountInventoryItemReq`; 348 `0x1400c9940` | ✅ `0x1400c5100` / `0x1401f9bc0` | ✅ 0 | ✅ `hqoccbqqi` @ `0x1401c3c90` |
| ✅ 91 | ✅ `HandleLobbyLobbyGate_RenameCharacterWithAccountInventoryItemReq`; 348 `0x1400c9fc0` | ✅ `0x1400c5780` / `0x1401f9bf0` | ✅ 0 | ✅ `hqosqqi` @ `0x1401c4548` |
| ✅ 94 | ✅ `HandleLobbyLobbyGate_AddSlotWithAccountInventoryItemReq`; 348 `0x1400ca560` | ✅ `0x1400c5d20` / `0x1401f9c20` | ✅ 0 | ✅ `hqqqi` @ `0x14017d720` |
| ✅ 97 | ✅ `HandleLobbyLobbyGate_ChangeUserNameWithAccountInventoryItemReq`; 348 `0x1400ca8b0` | ✅ `0x1400c6070` / `0x1401f9c50` | ✅ 0 | ✅ `hqsqqi` @ `0x1401ba49c` |
| ✅ 100 | ✅ `HandleLobbyLobbyGate_ExtendJumpingCharacterCreationQuotaWithAccountInventoryItemReq`; 348 `0x1400cac70` | ✅ `0x1400c6430` / `0x1401f9c80` | ✅ 0 | ✅ `hqqqi` @ `0x14017d720` |
| ✅ 103 | ✅ `HandleLobbyLobbyGate_CreateInitialUserNameReq`; 348 `0x1400cafd0` | ✅ `0x1400c6790` / `0x1401f9cb0` | ✅ 0 | ✅ `hqs` @ `0x1401ba714` |
| ✅ 106 | ✅ `HandleLobbyLobbyGate_ActivateGradeReq`; 348 `0x1400cb330` | ✅ `0x1400c6af0` / `0x1401f9ce0` | ✅ 0 | ✅ `hqQi` @ `0x140179668` |

### InfoGate

| Command | 348 handler name | 307 handler / slot | Fragment flag | Referenced format literals in 307 |
| --- | --- | --- | --- | --- |
| ✅ 1 | ❓ `HandleLobbyInfoGate_HelloReq`; 348 `0x140056d40` | ✅ `0x140055860` / `0x1401f9d40` | ✅ 0 | ✅ `hhhh` @ `0x14012c4ec`; ✅ `hh` @ `0x14012e610`; ✅ `hiChhQ;` @ `0x14015a090` |
| ✅ 4 | ❓ `HandleLobbyInfoGate_AuthTokenReq`; 348 `0x140057350` | ✅ `0x140055e70` / `0x1401f9d70` | ✅ 0 | ✅ `hqqo` @ `0x140140530` |
| ✅ 7 | ❓ `HandleLobbyInfoGate_GetWorldInfoListReq`; 348 `0x140057880` | ✅ `0x1400563a0` / `0x1401f9da0` | ✅ 0 | ✅ `hiChhQ;` @ `0x14015a090`; ✅ `h` @ `0x14015aa94` |

### GameDaemon

| Command | 348 handler name | 307 handler / slot | Fragment flag | Referenced format literals in 307 |
| --- | --- | --- | --- | --- |
| ✅ 1 | ✅ `HandleLobbyGame_HelloReq`; 348 `0x140088260` | ✅ `0x140086d50` / `0x1401f9df0` | ✅ 0 | ✅ `hh` @ `0x14012e610`; ✅ `hhhhh` @ `0x14017bd28`; ✅ `hCQiiiqqChii;;qqqi` @ `0x14017c898` |
| ✅ 4 | ❓ `HandleLobbyGame_ServiceOpenNotify`; 348 `0x140088cd0` | ✅ `0x1400877b0` / `0x1401f9e20` | ✅ 0 | ✅ `hhh` @ `0x1401608f0` |
| ✅ 5 | ❓ `HandleLobbyGame_EnterWorldReq`; 348 `0x140089270` | ✅ `0x140087d40` / `0x1401f9e30` | ✅ 0 | ✅ `hqqqi` @ `0x14017d720` |
| ✅ 8 | ❓ `HandleLobbyGame_LeaveWorldReq`; 348 `0x1400899a0` | ✅ `0x140088470` / `0x1401f9e60` | ✅ 0 | ✅ `hqqq` @ `0x14017dffc` |
| ✅ 11 | ❓ `HandleLobbyGame_UpdateBotScoreNotify`; 348 `0x140089d90` | ✅ `0x140088860` / `0x1401f9e90` | ✅ 0 | ✅ `hqqi` @ `0x14017e5ec` |
| ✅ 12 | ❓ `HandleLobbyGame_LogoutNotify`; 348 `0x14008a060` | ✅ `0x140088b30` / `0x1401f9ea0` | ✅ 0 | ✅ `hqq` @ `0x140161e14` |
| ✅ 15 | ❓ `HandleLobbyGame_KickOk`; 348 `0x14008a400` | ✅ `0x140088ed0` / `0x1401f9ed0` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 16 | ❓ `HandleLobbyGame_KickFail`; 348 `0x14008a560` | ✅ `0x140089030` / `0x1401f9ee0` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 17 | ❓ `HandleLobbyGame_NPPresenceKickReq`; 348 `0x140088f30` | ✅ `0x140087a10` / `0x1401f9ef0` | ✅ 0 | ✅ `hqQQ` @ `0x14017d20c` |
| ✅ 21 | ✅ `HandleLobbyGame_CreateCharacterOk`; 348 `0x14008a7c0` | ✅ `0x140089290` / `0x1401f9f30` | ✅ 0 | ✅ `hqhhb` @ `0x14017f35c` |
| ✅ 22 | ❓ `HandleLobbyGame_CreateCharacterFail`; 348 `0x14008aad0` | ✅ `0x1400895a0` / `0x1401f9f40` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 24 | ❓ `HandleLobbyGame_DeleteCharacterOk`; 348 `0x14008ad30` | ✅ `0x140089800` / `0x1401f9f60` | ✅ 0 | ✅ `hqh` @ `0x14017f824` |
| ✅ 25 | ❓ `HandleLobbyGame_DeleteCharacterFail`; 348 `0x14008af90` | ✅ `0x140089a60` / `0x1401f9f70` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 27 | ❓ `HandleLobbyGame_RestoreCharacterOk`; 348 `0x14008b1f0` | ✅ `0x140089cc0` / `0x1401f9f90` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 28 | ❓ `HandleLobbyGame_RestoreCharacterFail`; 348 `0x14008b350` | ✅ `0x140089e20` / `0x1401f9fa0` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 30 | ❓ `HandleLobbyGame_StartChargeReq`; 348 `0x14008b5b0` | ✅ `0x14008a080` / `0x1401f9fc0` | ✅ 0 | ✅ `hqqq` @ `0x14017dffc` |
| ✅ 33 | ✅ `HandleLobbyGame_StopChargeReq`; 348 `0x14008b9b0` | ✅ `0x14008a480` / `0x1401f9ff0` | ✅ 0 | ✅ `hqqqs` @ `0x140180598` |
| ✅ 38 | ❓ `HandleLobbyGame_AbandonChargeNotify`; 348 `0x14008be60` | ✅ `0x14008a930` / `0x1401fa040` | ✅ 0 | ✅ `hqqs` @ `0x140180bb4` |
| ✅ 39 | ✅ `HandleLobbyGame_UpdateCharacterDataReq`; 348 `0x14008c250` | ✅ `0x14008ad20` / `0x1401fa050` | ✅ 0 | ✅ `hqqqhb` @ `0x140181118` |
| ✅ 42 | ❓ `HandleLobbyGame_UpdateAccountDataReq`; 348 `0x14008c660` | ✅ `0x14008b130` / `0x1401fa080` | ✅ 0 | ✅ `hqqqiCQiqqihcc;qiq` @ `0x140181770` |
| ✅ 45 | ❓ `HandleLobbyGame_TryRegisterGuildReq`; 348 `0x14008d590` | ✅ `0x14008b770` / `0x1401fa0b0` | ✅ 0 | ✅ `hqcs` @ `0x1401820f0` |
| ✅ 48 | ❓ `HandleLobbyGame_FinishRegisterGuildNotify`; 348 `0x14008d9a0` | ✅ `0x14008bb80` / `0x1401fa0e0` | ✅ 0 | ✅ `hqq` @ `0x140161e14` |
| ✅ 49 | ❓ `HandleLobbyGame_CancelRegisterGuildNotify`; 348 `0x14008db10` | ✅ `0x14008bcf0` / `0x1401fa0f0` | ✅ 0 | ✅ `hqq` @ `0x140161e14` |
| ✅ 50 | ✅ `HandleLobbyGame_TryRenameGuildReq`; 348 `0x14008dc80` | ✅ `0x14008be60` / `0x1401fa100` | ✅ 0 | ✅ `hqqcs` @ `0x140182a7c` |
| ✅ 53 | ❓ `HandleLobbyGame_FinishRenameGuildNotify`; 348 `0x14008e010` | ✅ `0x14008c1f0` / `0x1401fa130` | ✅ 0 | ✅ `hqqq` @ `0x14017dffc` |
| ✅ 54 | ❓ `HandleLobbyGame_CancelRenameGuildNotify`; 348 `0x14008e220` | ✅ `0x14008c400` / `0x1401fa140` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 55 | ❓ `HandleLobbyGame_DeleteGuildNotify`; 348 `0x14008e390` | ✅ `0x14008c570` / `0x1401fa150` | ✅ 0 | ✅ `hqq` @ `0x140161e14` |
| ✅ 58 | ❓ `HandleLobbyGame_ChangeCharacterAppearanceOk`; 348 `0x14008e520` | ✅ `0x14008c700` / `0x1401fa180` | ✅ 0 | ✅ `hqb` @ `0x14018393c` |
| ✅ 59 | ❓ `HandleLobbyGame_ChangeCharacterAppearanceFail`; 348 `0x14008e820` | ✅ `0x14008ca00` / `0x1401fa190` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 61 | ❓ `HandleLobbyGame_ChangeCharacterSexOk`; 348 `0x14008eab0` | ✅ `0x14008cc90` / `0x1401fa1b0` | ✅ 0 | ✅ `hqb` @ `0x14018393c` |
| ✅ 62 | ❓ `HandleLobbyGame_ChangeCharacterSexFail`; 348 `0x14008edb0` | ✅ `0x14008cf90` / `0x1401fa1c0` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 64 | ❓ `HandleLobbyGame_ChangeCharacterRaceOk`; 348 `0x14008f040` | ✅ `0x14008d220` / `0x1401fa1e0` | ✅ 0 | ✅ `hqb` @ `0x14018393c` |
| ✅ 65 | ❓ `HandleLobbyGame_ChangeCharacterRaceFail`; 348 `0x14008f340` | ✅ `0x14008d520` / `0x1401fa1f0` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 67 | ❓ `HandleLobbyGame_RenameCharacterOk`; 348 `0x14008f5d0` | ✅ `0x14008d7b0` / `0x1401fa210` | ✅ 0 | ✅ `hqq` @ `0x140161e14` |
| ✅ 68 | ❓ `HandleLobbyGame_RenameCharacterFail`; 348 `0x14008f870` | ✅ `0x14008da50` / `0x1401fa220` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 69 | ✅ `HandleLobbyGame_ExtendJumpingCharacterCreationQuotaWithAccountInventoryItemReq`; 348 `0x14008fb00` | ✅ `0x14008dce0` / `0x1401fa230` | ✅ 0 | ✅ `hqqqqqi` @ `0x140184d68` |
| ✅ 72 | ❓ `HandleLobbyGame_UpgradeJumpingCharacterStateWithCreationQuotaReq`; 348 `0x14008ffc0` | ✅ `0x14008e1a0` / `0x1401fa260` | ✅ 0 | ✅ `hqqq` @ `0x14017dffc` |
| ✅ 75 | ❓ `HandleLobbyGame_UpdateJumpingCharacterStateReq`; 348 `0x1400903d0` | ✅ `0x14008e5b0` / `0x1401fa290` | ✅ 0 | ✅ `hqqq` @ `0x14017dffc` |
| ✅ 78 | ✅ `HandleLobbyGame_SetAppDataNotify`; 348 `0x1400907e0` | ✅ `0x14008e9c0` / `0x1401fa2c0` | ✅ 0 | ✅ `hqCss;` @ `0x140185e1c`; ✅ `s` @ `0x1401d2050` |
| ✅ 79 | ✅ `HandleLobbyGame_FriendMessageNotify`; 348 `0x140090f70` | ✅ `0x14008f150` / `0x1401fa2d0` | ✅ 0 | ✅ `hqsCss;` @ `0x1401866a0`; ✅ `s` @ `0x1401d2050` |
| ✅ 80 | ❓ `HandleLobbyGame_SecondPasswordGetStatusReq`; 348 `0x140091ab0` | ✅ `0x14008fc90` / `0x1401fa2e0` | ✅ 0 | ✅ `hqqc` @ `0x140187698` |
| ✅ 83 | ✅ `HandleLobbyGame_SecondPasswordIsVerifiedReq`; 348 `0x140091e80` | ✅ `0x140090060` / `0x1401fa310` | ✅ 0 | ✅ `hqqcQ` @ `0x140187a8c` |
| ✅ 87 | ❓ `HandleLobbyGame_AuctionEscrowOk`; 348 `0x140092280` | ✅ `0x140090460` / `0x1401fa350` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 88 | ❓ `HandleLobbyGame_AuctionEscrowFail`; 348 `0x140092400` | ✅ `0x1400905e0` / `0x1401fa360` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 89 | ✅ `HandleLobbyGame_AuctionBidReq`; 348 `0x1400926a0` | ✅ `0x140090880` / `0x1401fa370` | ✅ 0 | ✅ `hqohq` @ `0x140188340` |
| ✅ 92 | ✅ `HandleLobbyGame_GetGradeListReq`; 348 `0x140092ca0` | ✅ `0x140090e80` / `0x1401fa3a0` | ✅ 0 | ✅ `hqQqi` @ `0x1401888a8` |
| ✅ 97 | ✅ `HandleLobbyGame_ActivateGradeReq`; 348 `0x140092f00` | ✅ `0x1400910e0` / `0x1401fa3f0` | ✅ 0 | ✅ `hqqQi` @ `0x140188bb4` |
| ✅ 100 | ✅ `HandleLobbyGame_DepositGameMileageReq`; 348 `0x140093540` | ✅ `0x140091720` / `0x1401fa420` | ✅ 0 | ✅ `hqQQqqoqqqqqsQccc` @ `0x140188f58` |
| ✅ 103 | ❓ `HandleLobbyGame_BoastEventInWorldNotify`; 348 `0x140094780` | ✅ `0x140092960` / `0x1401fa450` | ✅ 0 | ✅ `hhsh` @ `0x1401409d4` |
| ✅ 104 | ✅ `HandleLobbyGame_TakeAccountTimeBasedPlayRewardReq`; 348 `0x140094910` | ✅ `0x140092af0` / `0x1401fa460` | ✅ 0 | ✅ `hqqiQhq` @ `0x140189dc8` |
| ✅ 107 | ✅ `HandleLobbyGame_UpdateAccountExpReq`; 348 `0x140094df0` | ✅ `0x140092fd0` / `0x1401fa490` | ✅ 0 | ✅ `hqqqqiq` @ `0x14018a328` |

❓ Guide-only commands outside the checked target range:
- ❓ 110 `HandleLobbyGame_UpdateAccountContentQuotaReq` at 348 `0x1400951f0`; table `0x140202de0`. Target rejects this command at dispatcher `0x14007F920`.
- ❓ 113 `HandleLobbyGame_UpdateAccountRecycleTimeReq` at 348 `0x140095890`; table `0x140202de0`. Target rejects this command at dispatcher `0x14007F920`.

### AccountInventory

| Command | 348 handler name | 307 handler / slot | Fragment flag | Referenced format literals in 307 |
| --- | --- | --- | --- | --- |
| ✅ 2 | ❓ `HandleAccountInventoryLobby_HelloOk`; 348 `0x1400087d0` | ✅ `0x140007a90` / `0x1401fa4e0` | ✅ 0 | ❓ Not found |
| ✅ 3 | ❓ `HandleAccountInventoryLobby_HelloFail`; 348 `0x140008820` | ✅ `0x140007ae0` / `0x1401fa4f0` | ✅ 0 | ✅ `hh` @ `0x14012e610` |
| ✅ 5 | ❓ `HandleAccountInventoryLobby_GetGoodsCountOk`; 348 `0x1400088b0` | ✅ `0x140007b70` / `0x1401fa510` | ✅ 0 | ✅ `hqi` @ `0x14012e73c` |
| ✅ 6 | ❓ `HandleAccountInventoryLobby_GetGoodsCountFail`; 348 `0x140008ad0` | ✅ `0x140007d90` / `0x1401fa520` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 8 | ❓ `HandleAccountInventoryLobby_ListGoodsOk`; 348 `0x140008d00` | ✅ `0x140007fc0` / `0x1401fa540` | ✅ 1 | ✅ `hiqCqiQQqqssissbbCiiibb;;` @ `0x14012ebd8` |
| ✅ 9 | ❓ `HandleAccountInventoryLobby_ListGoodsFail`; 348 `0x140009550` | ✅ `0x140008810` / `0x1401fa550` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 11 | ✅ `HandleAccountInventoryLobby_TakeGoodsOk`; 348 `0x140009750` | ✅ `0x140008a10` / `0x1401fa570` | ✅ 1 | ✅ `hiqCqqqiQ;` @ `0x14012f690` |
| ✅ 12 | ❓ `HandleAccountInventoryLobby_TakeGoodsFail`; 348 `0x140009a60` | ✅ `0x140008d20` / `0x1401fa580` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 14 | ✅ `HandleAccountInventoryLobby_RetrieveLobbyServiceItemListOk`; 348 `0x140009c60` | ✅ `0x140008f20` / `0x1401fa5a0` | ✅ 1 | ✅ `hiqiCqqqiQ;` @ `0x14012fbc0` |
| ✅ 15 | ❓ `HandleAccountInventoryLobby_RetrieveLobbyServiceItemListFail`; 348 `0x140009f60` | ✅ `0x140009220` / `0x1401fa5b0` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 17 | ❓ `HandleAccountInventoryLobby_RetrieveSingleWarehouseItemOk`; 348 `0x14000a160` | ✅ `0x140009420` / `0x1401fa5d0` | ✅ 0 | ✅ `hqqqqiQ` @ `0x140130180` |
| ✅ 18 | ❓ `HandleAccountInventoryLobby_RetrieveSingleWarehouseItemFail`; 348 `0x14000a440` | ✅ `0x140009700` / `0x1401fa5e0` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 20 | ❓ `HandleAccountInventoryLobby_TryWarehouseItemConsumptionOk`; 348 `0x14000a680` | ✅ `0x140009940` / `0x1401fa600` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 21 | ❓ `HandleAccountInventoryLobby_TryWarehouseItemConsumptionFail`; 348 `0x14000a780` | ✅ `0x140009a40` / `0x1401fa610` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 23 | ❓ `HandleAccountInventoryLobby_FinishWarehouseItemConsumptionOk`; 348 `0x14000a9c0` | ✅ `0x140009c80` / `0x1401fa630` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 24 | ❓ `HandleAccountInventoryLobby_FinishWarehouseItemConsumptionFail`; 348 `0x14000aac0` | ✅ `0x140009d80` / `0x1401fa640` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 26 | ❓ `HandleAccountInventoryLobby_CancelWarehouseItemConsumptionOk`; 348 `0x14000ad00` | ✅ `0x140009fc0` / `0x1401fa660` | ✅ 0 | ✅ `H` @ `0x1401311c0` |
| ✅ 27 | ❓ `HandleAccountInventoryLobby_CancelWarehouseItemConsumptionFail`; 348 `0x14000ad40` | ✅ `0x14000a000` / `0x1401fa670` | ✅ 0 | ✅ `H` @ `0x140131280` |
| ✅ 29 | ❓ `HandleAccountInventoryLobby_ConsumeWarehouseItemOk`; 348 `0x14000ad80` | ✅ `0x14000a040` / `0x1401fa690` | ✅ 0 | ✅ `hq` @ `0x140130828` |
| ✅ 30 | ❓ `HandleAccountInventoryLobby_ConsumeWarehouseItemFail`; 348 `0x14000ae80` | ✅ `0x14000a140` / `0x1401fa6a0` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 32 | ❓ `HandleAccountInventoryLobby_RetrieveCharacterGoodsCountOk`; 348 `0x14000b0c0` | ✅ `0x14000a380` / `0x1401fa6c0` | ✅ 0 | ✅ `hqii` @ `0x140131748` |
| ✅ 33 | ❓ `HandleAccountInventoryLobby_RetrieveCharacterGoodsCountFail`; 348 `0x14000b310` | ✅ `0x14000a5d0` / `0x1401fa6d0` | ✅ 0 | ✅ `hqE` @ `0x14012e984` |
| ✅ 34 | ✅ `HandleAccountInventoryLobby_GoodsInventoryChangedNotify`; 348 `0x14000b510` | ✅ `0x14000a7d0` / `0x1401fa6e0` | ✅ 1 | ✅ `hioQCqqi;Qqq` @ `0x140131cc8` |

## Other links

| Link | State | Source |
| --- | --- | --- |
| PostOffice | ❓ Dispatch table and commands not mapped | ✅ Startup link at 307 0x14009A830; ❓ guide 0x14009DCA0 |
| NP / StsMsg | ❓ Message registry and ABI not mapped; no ordinary packet table assumed | ✅ 307 0x14009B930 and 0x1400CB380; ❓ guide 0x14009F010 |
| DB | ✅ ODBC call inventory is in 03-db.md; not a packet command link | ✅ 307 0x1400E3250; ❓ guide 0x1400E7DF0 |

## LobbyGate check against ../04-dispatch.md

✅ All 28 client-to-daemon relay destinations in spec 04 section 4 have a non-null target handler. Source: 307 table 0x1401F9640.
✅ Daemon command 41 has CheckCharacterNameReq, although the gate-side client command 41 path is unreachable in spec 04. Source: target slot 0x1401F98D0.
✅ Command 0 has no handler and fails the target dispatcher range check. Thus the spec 04 client-command-33 mapping quirk reaches a rejected command, not a valid daemon command. Source: 307 0x1400BC750.
✅ The gate-side reply table in spec 04 section 3.1 is the opposite direction. It is not the daemon receive table.
✅ HelloReq decodes `hhhh` and requires packet version 58. Source: 307 0x1400BDC90; format 0x14012C4EC.
❓ No full reply-command or reply-body comparison was completed against spec 04 or spec 05. The 118 inbound handlers do not prove wire compatibility.

## Open checks

1. Verify every likely handler name against body evidence.
2. Separate receive formats from compose formats.
3. Expand fragment formats and assign field types.
4. Map PostOffice and NP registries.
5. Check all replies against the existing LobbyGate specs.

## LobbyGate link, full

✅ Build 0.0.307. Each of the 35 identities now has a command slot and an operation-specific log anchor. Identity is exact; this does not prove every job branch. Sources: handler addresses below and table `0x1401F9640`.

### Receive bodies

✅ The body excludes the first `h` (command). Formats below show the full codec literal to keep the source unambiguous. `hi` marks a fragment header in replies. `o` is 16 bytes; `q` is 8 bytes; `i` is 4 bytes; `h` is 2 bytes; `Q`/`c` are 1 byte. ❓ Signedness and bounds need a codec pass. `s` and `b` are variable text and blob fields.

| Command | Exact identity (prefix HandleLobbyLobbyGate_) | Receive literal | Reply commands | 307 source |
| --- | --- | --- | --- | --- |
| ✅ 1 | ✅ `HelloReq` | ✅ `hhhh` | ❓ 2 / 3 | ✅ `0x1400bdc90` |
| ✅ 6 | ✅ `InitSessionWithNPTokenReq` | ✅ `hqooi` | ❓ 7 / 8 | ✅ `0x1400be4c0` |
| ✅ 9 | ✅ `StartSessionWithNPTokenReq` | ✅ `hqoo` | ❓ 10 / 11 | ✅ `0x1400beab0` |
| ✅ 12 | ✅ `InitSessionWithLobbyTokenReq` | ✅ `hqqqi` | ❓ 13 / 14 | ✅ `0x1400befc0` |
| ✅ 15 | ✅ `StartSessionWithLobbyTokenReq` | ✅ `hq` | ❓ 16 / 17 | ✅ `0x1400bf5a0` |
| ✅ 18 | ✅ `LostAnonymousConnectionNotify` | ✅ `hq` | ❓ none directly | ✅ `0x1400bf8b0` |
| ✅ 19 | ✅ `LostConnectionNotify` | ✅ `hq` | ❓ none directly | ✅ `0x1400bfb20` |
| ✅ 23 | ✅ `LogoutNotify` | ✅ `hq` | ❓ none directly | ✅ `0x1400bfd80` |
| ✅ 24 | ✅ `GetEnterWorldTokenReq` | ✅ `hqo` | ❓ 25 / 26 / 27 | ✅ `0x1400bffe0` |
| ✅ 28 | ✅ `GetWaitInfoReq` | ✅ `hq` | ❓ 29 / 30 | ✅ `0x1400c0520` |
| ✅ 31 | ✅ `GiveupWaitReq` | ✅ `hq` | ❓ 32 / 33 | ✅ `0x1400c0770` |
| ✅ 35 | ✅ `GetCharacterInfoListReq` | ✅ `hq` | ❓ 36 / 37 | ✅ `0x1400c0a90` |
| ✅ 38 | ✅ `GetCharacterInfoReq` | ✅ `hqo` | ❓ 39 / 40 | ✅ `0x1400c0dd0` |
| ✅ 41 | ✅ `CheckCharacterNameReq` | ✅ No decode; N/A stub | ❓ none; N/A stub | ✅ `0x1400c1270` |
| ✅ 44 | ✅ `CreateCharacterReq` | ✅ `hqohsbQ` | ❓ 45 / 46 | ✅ `0x1400c12a0` |
| ✅ 47 | ✅ `DeleteCharacterReq` | ✅ `hqo` | ❓ 48 / 49; notify 50 | ✅ `0x1400c1920` |
| ✅ 51 | ✅ `CancelWaitCharacterDeletionReq` | ✅ `hqo` | ❓ 52 / 53 | ✅ `0x1400c1f60` |
| ✅ 54 | ✅ `RecoverCharacterReq` | ✅ `hqo` | ❓ 55 / 56 | ✅ `0x1400c25a0` |
| ✅ 57 | ✅ `BindCharacterSlotReq` | ✅ `hqoq` | ❓ ❓ success 58; failure 59 | ✅ `0x1400c2ca0` |
| ✅ 60 | ✅ `GetChargeInfoReq` | ✅ `hq` | ❓ 61 / 62; notify 63 | ✅ `0x1400c3250` |
| ✅ 64 | ✅ `GetGoodsCountReq` | ✅ `hq` | ❓ 65 / 66 | ✅ `0x1400c3550` |
| ✅ 67 | ✅ `ListGoodsReq` | ✅ `hqc` | ❓ 68 / 69 | ✅ `0x1400c37e0` |
| ✅ 70 | ✅ `TakeGoodsReq` | ✅ `hqq` | ❓ 71 / 72 | ✅ `0x1400c3b30` |
| ✅ 73 | ✅ `ListLobbyServiceItemInfoReq` | ✅ `hqQq` | ❓ 74 / 75 | ✅ `0x1400c3d40` |
| ✅ 76 | ✅ `GetLobbyServiceItemInfoReq` | ✅ `hqq` | ❓ 77 / 78 | ✅ `0x1400c3fc0` |
| ✅ 79 | ✅ `GetServiceEntryListReq` | ✅ `hq` | ❓ 80 / 81 | ✅ `0x1400c4200` |
| ✅ 82 | ✅ `ChangeCharacterAppearanceWithAccountInventoryItemReq` | ✅ `hqobqqi` | ❓ 83 / 84 | ✅ `0x1400c44c0` |
| ✅ 85 | ✅ `ChangeCharacterSexWithAccountInventoryItemReq` | ✅ `hqocbqqi` | ❓ 86 / 87 | ✅ `0x1400c4ac0` |
| ✅ 88 | ✅ `ChangeCharacterRaceWithAccountInventoryItemReq` | ✅ `hqoccbqqi` | ❓ 89 / 90 | ✅ `0x1400c5100` |
| ✅ 91 | ✅ `RenameCharacterWithAccountInventoryItemReq` | ✅ `hqosqqi` | ❓ 92 / 93 | ✅ `0x1400c5780` |
| ✅ 94 | ✅ `AddSlotWithAccountInventoryItemReq` | ✅ `hqqqi` | ❓ 95 / 96 | ✅ `0x1400c5d20` |
| ✅ 97 | ✅ `ChangeUserNameWithAccountInventoryItemReq` | ✅ `hqsqqi` | ❓ 98 / 99 | ✅ `0x1400c6070` |
| ✅ 100 | ✅ `ExtendJumpingCharacterCreationQuotaWithAccountInventoryItemReq` | ✅ `hqqqi` | ❓ 101 / 102 | ✅ `0x1400c6430` |
| ✅ 103 | ✅ `CreateInitialUserNameReq` | ✅ `hqs` | ❓ 104 / 105 | ✅ `0x1400c6790` |
| ✅ 106 | ✅ `ActivateGradeReq` | ✅ `hqQi` | ❓ 107 / 108; notify 109 / 110 | ✅ `0x1400c6af0` |

❓ Reply families above use command numbering and the checked sender inventory. Job-to-sender reachability is not complete. Do not treat this table as a complete branch specification.

### Checked sender inventory

✅ Each row gives an emitted command and its compose format. No identity was inferred from function order. Sources are 307 functions. ❓ This inventory is not proved exhaustive outside the checked channel range.

| Command | Full compose literal | Sender |
| --- | --- | --- |
| ✅ 5 | ✅ `hs` | ✅ `0x1400bc2a0` |
| ✅ 7 | ✅ `hqqoo` | ✅ `0x1400b7570` |
| ✅ 8 | ✅ `hqE` | ✅ `0x1400b7680` |
| ✅ 10 | ✅ `hiqQoihCiiiii;CiiQiqq;Qqiqiq` | ✅ `0x1400b78f0` |
| ✅ 11 | ✅ `hqE` | ✅ `0x1400b7c00` |
| ✅ 13 | ✅ `hqq` | ✅ `0x1400b7750` |
| ✅ 14 | ✅ `hqE` | ✅ `0x1400b7820` |
| ✅ 16 | ✅ `hiqCiiiii;CiiQiqq;iqiq` | ✅ `0x1400b7cd0` |
| ✅ 17 | ✅ `hqE` | ✅ `0x1400b7f90` |
| ✅ 20 | ✅ `hq` | ✅ `0x1400b8060` |
| ✅ 21 | ✅ `hq` | ✅ `0x1400b8120` |
| ✅ 22 | ✅ `hqQ` | ✅ `0x1400b81e0` |
| ✅ 25 | ✅ `hqqh` | ✅ `0x1400b8c60` |
| ✅ 26 | ✅ `hqE` | ✅ `0x1400b8b90` |
| ✅ 27 | ✅ `hqiiCch;` | ✅ `0x1400b8d40` |
| ✅ 29 | ✅ `hqiiCch;` | ✅ `0x1400b8ff0` |
| ✅ 30 | ✅ `hqE` | ✅ `0x1400b92a0` |
| ✅ 32 | ✅ `hq` | ✅ `0x1400b9370` |
| ✅ 33 | ✅ `hqE` | ✅ `0x1400b9430` |
| ✅ 34 | ✅ `hqq` | ✅ `0x1400b9500` |
| ✅ 37 | ✅ `hqE` | ✅ `0x1400b82b0` |
| ✅ 40 | ✅ `hqoE` | ✅ `0x1400b8380` |
| ✅ 45 | ✅ `hqqohsbqQQii` | ✅ `0x1400b8470` |
| ✅ 46 | ✅ `hqoE` | ✅ `0x1400b85e0` |
| ✅ 48 | ✅ `hqoQi` | ✅ `0x1400b86d0` |
| ✅ 49 | ✅ `hqoQE` | ✅ `0x1400b87d0` |
| ✅ 50 | ✅ `hqo` | ✅ `0x1400b8ac0` |
| ✅ 52 | ✅ `hqo` | ✅ `0x1400b88e0` |
| ✅ 53 | ✅ `hqoQE` | ✅ `0x1400b89b0` |
| ✅ 55 | ✅ `hqo` | ✅ `0x1400b97e0` |
| ✅ 56 | ✅ `hqoQE` | ✅ `0x1400b98b0` |
| ✅ 59 | ✅ `hqoqE` | ✅ `0x1400b99c0` |
| ✅ 61 | ✅ `hqQQihQiQQQ` | ✅ `0x1400b95d0` |
| ✅ 62 | ✅ `hqE` | ✅ `0x1400b9710` |
| ✅ 63 | ✅ `hqQq` | ✅ `0x1400b9ac0` |
| ✅ 65 | ✅ `hqi` | ✅ `0x1400b9ba0` |
| ✅ 66 | ✅ `hqE` | ✅ `0x1400b9c60` |
| ✅ 68 | ✅ `hiqCqiQQqqssissbbCiiibb;;` | ✅ `0x1400b9d30` |
| ✅ 69 | ✅ `hqE` | ✅ `0x1400b9f40` |
| ✅ 71 | ✅ `hiqqCqqqiQ;` | ✅ `0x1400ba010` |
| ✅ 72 | ✅ `hqE` | ✅ `0x1400ba2b0` |
| ✅ 74 | ✅ `hiqQiCqqqiQ;` | ✅ `0x1400ba380` |
| ✅ 75 | ✅ `hqQE` | ✅ `0x1400ba610` |
| ✅ 77 | ✅ `hqqqqiQ` | ✅ `0x1400ba6f0` |
| ✅ 78 | ✅ `hqqE` | ✅ `0x1400ba830` |
| ✅ 81 | ✅ `hqE` | ✅ `0x1400ba910` |
| ✅ 83 | ✅ `hqobqq` | ✅ `0x1400ba9e0` |
| ✅ 84 | ✅ `hqoqqE` | ✅ `0x1400baae0` |
| ✅ 86 | ✅ `hqobqq` | ✅ `0x1400babe0` |
| ✅ 87 | ✅ `hqoqqE` | ✅ `0x1400bace0` |
| ✅ 89 | ✅ `hqobqq` | ✅ `0x1400bade0` |
| ✅ 90 | ✅ `hqoqqE` | ✅ `0x1400baee0` |
| ✅ 92 | ✅ `hqosqq` | ✅ `0x1400bafe0` |
| ✅ 93 | ✅ `hqoqqE` | ✅ `0x1400bb0f0` |
| ✅ 95 | ✅ `hqqqo` | ✅ `0x1400bb1f0` |
| ✅ 96 | ✅ `hqqqE` | ✅ `0x1400bb2f0` |
| ✅ 98 | ✅ `hqsqqi` | ✅ `0x1400bb3d0` |
| ✅ 99 | ✅ `hqqqE` | ✅ `0x1400bb510` |
| ✅ 101 | ✅ `hqqqi` | ✅ `0x1400bb620` |
| ✅ 102 | ✅ `hqqqE` | ✅ `0x1400bb700` |
| ✅ 104 | ✅ `hqs` | ✅ `0x1400bb7e0` |
| ✅ 105 | ✅ `hqE` | ✅ `0x1400bb900` |
| ✅ 107 | ✅ `hqiiqq` | ✅ `0x1400bb9f0` |
| ✅ 108 | ✅ `hqQiE` | ✅ `0x1400bbb10` |
| ✅ 109 | ✅ `hqiiQiqqQ` | ✅ `0x1400bbc20` |
| ✅ 110 | ✅ `hqiiiii` | ✅ `0x1400bbe00` |
| ✅ 2 / 3 | ✅ `hiChhsQQiihhiQQQQQQQ;QQqCqqiQiic;` / `hh` | ✅ `0x1400BDC90` |
| ✅ 36 | ✅ `hiqqhCqohsbqQQi;` | ✅ `0x1400C0A90` |
| ✅ 39 | ✅ `hqqohsbqQQi` | ✅ `0x1400C0DD0` |
| ✅ 80 | ✅ `hiqCqQqohq;` | ✅ `0x1400C4200` |

### Cross-check against gate specs 04 and 05

| Scope | Result | Sources |
| --- | --- | --- |
| Hello, NP and lobby-token request/reply literals | ✅ No format difference found | 307 handlers 1/6/9/12/15 and senders 7/8/10/11/13/14/16/17 above; gate 05 §§1–2 |
| Close and kick 20/21/22 | ✅ Same `hq`, `hq`, `hqQ` | 307 `0x1400B8060` / `0x1400B8120` / `0x1400B81E0`; gate 05 §6.3 |
| Enter-world 25 and character list 36 | ✅ Same `hqqh` and `hiqqhCqohsbqQQi;` | 307 `0x1400B8C60` / `0x1400C0A90`; gate 05 §7 |
| Goods replies 68/71/74 | ✅ Same full fragment formats | 307 `0x1400B9D30` / `0x1400BA010` / `0x1400BA380`; gate 05 §§8.7–8.9 |
| Relay requests | ✅ All 28 destinations exist. ❓ Gate relay does not specify every body | 307 `0x1401F9640`; gate 04 §4 |
| Commands 0 and 41 | ✅ Keep the known gate quirks. Command 0 fails dispatch. Command 41 only logs N/A | 307 `0x1400BC750` / `0x1400C1270`; gate 04 §§4–5 |
| Other bypass replies | ❓ Gate spec 04 strips `hq` only. No independent semantic body comparison is available there | Sender inventory above; gate 04 §5 |
| Sender command 58; world update 4 | ❓ Not located in this pass | Command range 307 `0x1400BC750`; gate 04 §3.1 |

❓ No format mismatch was found in the compared formats. This is not a proof for unchecked bypass bodies or fragment split points.
