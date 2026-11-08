# 01 — Configuration survey

✅ marks a target code load. ❓ marks a guide field identifier or an unverified XML option spelling.
No endpoint, credential, connection-string or key value is recorded.
The XML value extraction command required approval and did not run. A names-only search is separate.
❓ A complete XML schema, comments and default-value pass remains open.

## Code-derived configuration fields

All offsets are relative to the config pointer at daemon +1864.
Field identifiers below come from the guide. They are not confirmed XML option spellings.

| Guide field identifier | Target type/load | Default | Code source |
| --- | --- | --- | --- |
| ❓ workerThreadCount | ✅ u32 at +40 | ❓ unchecked | 307 0x14009A0B0; guide 0x14009D520 |
| ❓ timerInterval | ✅ u32 at +48 | ❓ unchecked | 307 0x140097B10; guide 0x14009AF60 |
| ❓ timerMaxTime | ✅ u32 at +52 | ❓ unchecked | 307 0x140097B20; guide 0x14009AF70 |
| ❓ maxNPSessionCount | ✅ i32 at +88 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ maxLobbyGateSessionCount | ✅ i32 at +92 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ maxInfoGateSessionCount | ✅ i32 at +96 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ maxGameWorldSessionCount | ✅ i32 at +100 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ maxManageSessionCount | ✅ i32 at +104 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ maxPostOfficeSessionCount | ✅ i32 at +108 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ npAddr | ✅ u32 at +184 | Not recorded | 307 0x14009B930 |
| ❓ npPort | ✅ u16 at +188 | ❓ unchecked | 307 0x14009B930 |
| ❓ worldPort | ✅ u16 at +192 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ lobbyGatePort | ✅ u16 at +196 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ infoGatePort | ✅ u16 at +200 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ managePort | ✅ u16 at +204 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ useAccountInventory | ✅ byte flag at +374 | ❓ unchecked | 307 0x14009A0B0; 0x14009A830 |
| ❓ accountInventoryAddr | ✅ u32 at +376 | Not recorded | 307 0x14009A830 |
| ❓ accountInventoryPort | ✅ u16 at +380 | ❓ unchecked | 307 0x14009A830 |
| ❓ accountInventoryMaxSessionCount | ✅ i32 at +384 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ useInfoGate | ✅ byte flag at +388 | ❓ unchecked | 307 0x14009A0B0 |
| ❓ enableAuctionEscrowService | ✅ byte flag at +520 | ❓ unchecked | 307 0x14009A0B0; 0x14009A830 |

✅ Most listener session limits become zero when the configured count is non-positive. Positive counts add one. Source: 307 0x14009A0B0.
✅ The AccountInventory limit branch tests config +384 but uses config +88 plus one on the positive path. Source: 307 0x14009A0B0. This is also visible in guide 0x14009D520.
✅ Config validation logs use LobbyConfig.cpp at 0x140192EA8, referenced by 0x140095510.
❓ Validation rules, option-definition arrays, XML parser calls and defaults are not mapped in this job.
❓ ContentsRule, common config and service-list schemas remain open. No client data-file research was done.

## XML option names — step 2

✅ Names below were extracted without values from the 2020 sample set. XML presence alone does not prove 307 code support. Therefore each schema row is ❓ until its option descriptor is matched to the target code. Source paths and line numbers refer to the supplied samples. No values, certificates, account data or endpoints are copied.

❓ Defaults remain unchecked for every row. A value in a sample is not a code default. Empty field columns mean that no match was checked. Options `cert` and `tencent-uic-*` are presence-only boundaries, not implementation work. No comments were needed for this name inventory.

| XML option name | Proposed code field / type | Code evidence | XML source | Default |
| --- | --- | --- | --- | --- |
| ❓ `account-hash-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:92` | ❓ Unchecked |
| ❓ `account-inventory-adress` | ❓ config +376; IPv4 | ✅ Load at `0x14009A830`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:210` | ❓ Unchecked |
| ❓ `account-inventory-max-session-count` | ❓ config +384; i32 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:214` | ❓ Unchecked |
| ❓ `account-inventory-port` | ❓ config +380; u16 | ✅ Load at `0x14009A830`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:212` | ❓ Unchecked |
| ❓ `activity-pre-resting-status-period` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:120` | ❓ Unchecked |
| ❓ `activity-resting-status-period` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:118` | ❓ Unchecked |
| ❓ `add-slot-item-data-ids` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:270` | ❓ Unchecked |
| ❓ `allow-client-ip-address-in-logs` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:338` | ❓ Unchecked |
| ❓ `allowed-character-name-space-amount` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:140` | ❓ Unchecked |
| ❓ `allowed-guild-name-space-amount` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:180` | ❓ Unchecked |
| ❓ `allowed-max-character-name-length` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:138` | ❓ Unchecked |
| ❓ `allowed-max-guild-name-length` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:178` | ❓ Unchecked |
| ❓ `allowed-max-user-name-length` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:200` | ❓ Unchecked |
| ❓ `allowed-min-character-name-length` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:136` | ❓ Unchecked |
| ❓ `allowed-min-guild-name-length` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:176` | ❓ Unchecked |
| ❓ `allowed-min-user-name-length` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:198` | ❓ Unchecked |
| ❓ `apply-charge` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:228` | ❓ Unchecked |
| ❓ `auction-escrow-min-money` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:280` | ❓ Unchecked |
| ❓ `auction-escrow-money-item-key` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:284` | ❓ Unchecked |
| ❓ `auction-escrow-unit-money` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:282` | ❓ Unchecked |
| ❓ `cert` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:16` | ❓ Unchecked |
| ❓ `change-character-appearance-item-data-ids` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:258` | ❓ Unchecked |
| ❓ `change-character-race-item-data-ids` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:262` | ❓ Unchecked |
| ❓ `change-character-sex-item-data-ids` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:260` | ❓ Unchecked |
| ❓ `change-user-name-item-data-ids` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:274` | ❓ Unchecked |
| ❓ `character-deletion-scheduler-concurrent-processing-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:154` | ❓ Unchecked |
| ❓ `character-deletion-scheduler-period` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:152` | ❓ Unchecked |
| ❓ `character-name-reuse-limit-duration` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:142` | ❓ Unchecked |
| ❓ `check-character-goods-for-character-deletion` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:156` | ❓ Unchecked |
| ❓ `check-ip-mismatch` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:102` | ❓ Unchecked |
| ❓ `check-slot-capacity` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:276` | ❓ Unchecked |
| ❓ `cli-session-hash-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:84` | ❓ Unchecked |
| ❓ `config-path` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:10` | ❓ Unchecked |
| ❓ `content-ban-id-for-jumping-character` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:126` | ❓ Unchecked |
| ❓ `data-path` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:12` | ❓ Unchecked |
| ❓ `dummy-data` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/ContentsRule.lobbyd.config.xml:4` | ❓ Unchecked |
| ❓ `dump-type` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:22` | ❓ Unchecked |
| ❓ `enable-account-contents` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/ContentsRule.lobbyd.config.xml:13` | ❓ Unchecked |
| ❓ `enable-activity-status` | ❓ config +700; bool | ✅ Load at `0x1400CECD0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:116` | ❓ Unchecked |
| ❓ `enable-auction-escrow-service` | ❓ config +520; bool | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:278` | ❓ Unchecked |
| ❓ `enable-change-user-name-service` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:272` | ❓ Unchecked |
| ❓ `enable-content-ban` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:122` | ❓ Unchecked |
| ❓ `enable-friend-message-notification` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:108` | ❓ Unchecked |
| ❓ `enable-game-mileage` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:306` | ❓ Unchecked |
| ❓ `enable-game-time-schedule` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:128` | ❓ Unchecked |
| ❓ `enable-guild-join-renewal` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/ContentsRule.lobbyd.config.xml:15` | ❓ Unchecked |
| ❓ `enable-jumping-character-creation` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:168` | ❓ Unchecked |
| ❓ `enable-mileage` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:290` | ❓ Unchecked |
| ❓ `enable-mileage-batch-retry-on-startup` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:302` | ❓ Unchecked |
| ❓ `enable-mileage-deposit-notification` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:300` | ❓ Unchecked |
| ❓ `enable-multilingual-for-character-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:164` | ❓ Unchecked |
| ❓ `enable-multilingual-for-guild-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:188` | ❓ Unchecked |
| ❓ `enable-multilingual-for-user-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:204` | ❓ Unchecked |
| ❓ `enable-np-recv-completion-log` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:132` | ❓ Unchecked |
| ❓ `enable-numeric-for-character-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:160` | ❓ Unchecked |
| ❓ `enable-numeric-for-guild-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:184` | ❓ Unchecked |
| ❓ `enable-play-warrior` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/ContentsRule.lobbyd.config.xml:11` | ❓ Unchecked |
| ❓ `enable-reload-grade-while-return-to-lobby` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:250` | ❓ Unchecked |
| ❓ `enable-rms-inquiry` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:124` | ❓ Unchecked |
| ❓ `enable-second-password` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:112` | ❓ Unchecked |
| ❓ `export-lobby-stats-file-path` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:324` | ❓ Unchecked |
| ❓ `export-lobby-stats-file-period` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:322` | ❓ Unchecked |
| ❓ `extend-jumping-character-creation-quota-item-data-ids` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:286` | ❓ Unchecked |
| ❓ `forbidden-characters-for-character-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:166` | ❓ Unchecked |
| ❓ `forbidden-characters-for-guild-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:190` | ❓ Unchecked |
| ❓ `force-sentence-case-for-character-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:162` | ❓ Unchecked |
| ❓ `force-sentence-case-for-guild-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:186` | ❓ Unchecked |
| ❓ `force-sentence-case-for-user-name` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:202` | ❓ Unchecked |
| ❓ `free-play-type` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:230` | ❓ Unchecked |
| ❓ `friend-message-recipient-app-id-list` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:110` | ❓ Unchecked |
| ❓ `game-cash-virtual-currency-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:318` | ❓ Unchecked |
| ❓ `game-grade-key-for-get-level` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:244` | ❓ Unchecked |
| ❓ `game-mileage-deposit-reason-code` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:312` | ❓ Unchecked |
| ❓ `game-mileage-deposit-request-code` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:310` | ❓ Unchecked |
| ❓ `game-mileage-effective-duration-in-days` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:314` | ❓ Unchecked |
| ❓ `game-mileage-virtual-currency-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:308` | ❓ Unchecked |
| ❓ `grade-level-group-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:242` | ❓ Unchecked |
| ❓ `grade-priority-queue-game-grade-key` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:248` | ❓ Unchecked |
| ❓ `guild-name-reuse-limit-duration` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:182` | ❓ Unchecked |
| ❓ `info-gate-port` | ❓ config +200; u16 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:220` | ❓ Unchecked |
| ❓ `info-gate-public-address` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:222` | ❓ Unchecked |
| ❓ `info-gate-public-port` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:224` | ❓ Unchecked |
| ❓ `init-client-session-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:86` | ❓ Unchecked |
| ❓ `init-lobby-job-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:90` | ❓ Unchecked |
| ❓ `initial-user-name-enable` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:194` | ❓ Unchecked |
| ❓ `initial-user-name-prefix-list` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:196` | ❓ Unchecked |
| ❓ `limit-not-charge-pc-cafe` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:234` | ❓ Unchecked |
| ❓ `lobby-gate-port` | ❓ config +196; u16 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:68` | ❓ Unchecked |
| ❓ `lobby-job-hash-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:88` | ❓ Unchecked |
| ❓ `lobby-job-transaction-life-time` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:94` | ❓ Unchecked |
| ❓ `lobby-token-life-time` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:96` | ❓ Unchecked |
| ❓ `lobby-world-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:82` | ❓ Unchecked |
| ❓ `log-path` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:8` | ❓ Unchecked |
| ❓ `long-term-character-deletion-criteria-level` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:150` | ❓ Unchecked |
| ❓ `long-term-character-deletion-time` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:148` | ❓ Unchecked |
| ❓ `manage-port` | ❓ config +204; u16 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:70` | ❓ Unchecked |
| ❓ `max-game-world-session-count` | ❓ config +100; i32 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:52` | ❓ Unchecked |
| ❓ `max-info-gate-session-count` | ❓ config +96; i32 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:50` | ❓ Unchecked |
| ❓ `max-lobby-gate-session-count` | ❓ config +92; i32 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:48` | ❓ Unchecked |
| ❓ `max-lobby-service-item-data-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:256` | ❓ Unchecked |
| ❓ `max-management-session-count` | ❓ config +104; i32 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:54` | ❓ Unchecked |
| ❓ `max-np-session-count` | ❓ config +88; i32 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:46` | ❓ Unchecked |
| ❓ `max-postoffice-session-count` | ❓ config +108; i32 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:56` | ❓ Unchecked |
| ❓ `max-slot-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:158` | ❓ Unchecked |
| ❓ `mileage-batch-retry-concurrent-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:304` | ❓ Unchecked |
| ❓ `mileage-deposit-period` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:296` | ❓ Unchecked |
| ❓ `mileage-deposit-reason` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:294` | ❓ Unchecked |
| ❓ `mileage-deposit-request-code` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:292` | ❓ Unchecked |
| ❓ `mileage-schedule-week-count` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:298` | ❓ Unchecked |
| ❓ `min-bot-score-to-apply-bot-priority-queue` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/ContentsRule.lobbyd.config.xml:7` | ❓ Unchecked |
| ❓ `min-lobby-service-item-data-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:254` | ❓ Unchecked |
| ❓ `np-address` | ❓ config +184; IPv4 | ✅ Load at `0x14009B930`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:60` | ❓ Unchecked |
| ❓ `np-app-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/bns.common.config.xml:12` | ❓ Unchecked |
| ❓ `np-client-app-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/bns.common.config.xml:14` | ❓ Unchecked |
| ❓ `np-game-code` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/bns.common.config.xml:10` | ❓ Unchecked |
| ❓ `np-ping-interval` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:64` | ❓ Unchecked |
| ❓ `np-port` | ❓ config +188; u16 | ✅ Load at `0x14009B930`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:62` | ❓ Unchecked |
| ❓ `np-program-id` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/bns.common.config.xml:8` | ❓ Unchecked |
| ❓ `p-type` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/bns.common.config.xml:4` | ❓ Unchecked |
| ❓ `permit-enter-world-in-change-character-appearance` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:264` | ❓ Unchecked |
| ❓ `post-office-address` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:72` | ❓ Unchecked |
| ❓ `post-office-port` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:74` | ❓ Unchecked |
| ❓ `ranking-service-mode` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/bns.common.config.xml:6` | ❓ Unchecked |
| ❓ `rename-character-by-system-item-data-ids` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:268` | ❓ Unchecked |
| ❓ `rename-character-item-data-ids` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:266` | ❓ Unchecked |
| ❓ `reserved-game-world-wid-range-max` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:36` | ❓ Unchecked |
| ❓ `reserved-game-world-wid-range-min` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:34` | ❓ Unchecked |
| ❓ `reserved-info-gate-wid-range-max` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:40` | ❓ Unchecked |
| ❓ `reserved-info-gate-wid-range-min` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:38` | ❓ Unchecked |
| ❓ `reserved-lobby-gate-wid-range-max` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:32` | ❓ Unchecked |
| ❓ `reserved-lobby-gate-wid-range-min` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:30` | ❓ Unchecked |
| ❓ `root-path` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:6` | ❓ Unchecked |
| ❓ `second-password-type` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:114` | ❓ Unchecked |
| ❓ `short-term-character-deletion-time` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:146` | ❓ Unchecked |
| ❓ `stop-play-game-world-timeout` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:100` | ❓ Unchecked |
| ❓ `store-game-account-name` | ❓ config +266; bool | ✅ Load at `0x14009D7C0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:106` | ❓ Unchecked |
| ❓ `store-user-ip-address` | ❓ config +265; bool | ✅ Load at `0x14009D7C0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:104` | ❓ Unchecked |
| ❓ `support-languages` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:78` | ❓ Unchecked |
| ❓ `teen-age-boundary` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:130` | ❓ Unchecked |
| ❓ `tencent-uic-config` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:332` | ❓ Unchecked |
| ❓ `tencent-uic-enable` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:330` | ❓ Unchecked |
| ❓ `tencent-uic-inverval` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:334` | ❓ Unchecked |
| ❓ `timer-interval` | ❓ config +48; u32 | ✅ Load at `0x140097B10`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:24` | ❓ Unchecked |
| ❓ `timer-max-time` | ❓ config +52; u32 | ✅ Load at `0x140097B20`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:26` | ❓ Unchecked |
| ❓ `trial-play-limit-level` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:232` | ❓ Unchecked |
| ❓ `type-for-failed-event-object-log` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:42` | ❓ Unchecked |
| ❓ `use-account-inventory` | ❓ config +374; bool | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:208` | ❓ Unchecked |
| ❓ `use-bot-priority-queue` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/ContentsRule.lobbyd.config.xml:6` | ❓ Unchecked |
| ❓ `use-grade` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:238` | ❓ Unchecked |
| ❓ `use-grade-level` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:240` | ❓ Unchecked |
| ❓ `use-grade-priority-queue` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:246` | ❓ Unchecked |
| ❓ `use-info-gate` | ❓ config +388; bool | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:218` | ❓ Unchecked |
| ❓ `use-jumping-character-creation-quota` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:170` | ❓ Unchecked |
| ❓ `use-jumping-character-creation-quota-in-world` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:172` | ❓ Unchecked |
| ❓ `use-perf-monitor` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:326` | ❓ Unchecked |
| ❓ `use-time-based-play-schedule` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/ContentsRule.lobbyd.config.xml:9` | ❓ Unchecked |
| ❓ `wait-character-deletion` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:144` | ❓ Unchecked |
| ❓ `wait-time-for-reuse-pool-object` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:28` | ❓ Unchecked |
| ❓ `worker-thread` | ❓ config +40; u32 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:20` | ❓ Unchecked |
| ❓ `world-info-list-update-period` | ❓ Not matched | ❓ Not checked | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:98` | ❓ Unchecked |
| ❓ `world-port` | ❓ config +192; u16 | ✅ Load at `0x14009A0B0`; ❓ name association | `BnS_2020/LobbyDaemon/config/lobbyd.config.xml:66` | ❓ Unchecked |

✅ Preserve the sample spellings `worker-thread`, `max-management-session-count`, `max-postoffice-session-count`, `account-inventory-adress` and `tencent-uic-inverval`. Do not silently replace them with inferred field spellings. Source: XML rows above. ❓ Whether the code accepts aliases is not checked.

✅ `ContentsRule.lobbyd.config.xml` has the same seven extracted names in bin and config. The common config has six names. Listener-list XML files have no matches for a `name` attribute in this extraction. ❓ Their element/attribute schemas remain unchecked. Source: the names-only search over both directories.
