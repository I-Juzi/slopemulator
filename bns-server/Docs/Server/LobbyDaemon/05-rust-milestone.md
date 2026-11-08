# 05 — Local Rust milestone

This file defines the new local implementation. It does not add original-server facts.
Source facts stay in 02-links.md, 03-db.md and 04-login.md.

| Rule | Local implementation contract |
| --- | --- |
| R1 | Accept trusted frames of 4..4000 bytes. Keep partial reads across writes. Require D1 before session commands. Require version 58. Send D2 with the checked complete format and empty world/schedule rows. Send D3 reason 2 for an invalid version. ❓ The original full Hello failure map remains open. |
| R2 | D6 requires a nonzero gateKey and two nonzero GUIDs. Send D7 with a fresh cliKey and echoed GUIDs. Do not consume the NP token yet. Duplicate gate bindings and local capacity failures send D8 reason 6. Malformed or zero fields send reason 2. |
| R3 | D9 requires the owning link, Init state and matching GUIDs. Missing session sends D11 reason 4. Invalid state or GUID mismatch sends reason 5. Bad body sends reason 2. GUID matching and link ownership are explicit local safety rules. |
| R4 | NP rejects unknown or consumed tokens. Token rejection maps to 15; missing fields/unavailable/timeout to 2; checked presence failures map to 18 and 16. Each NP operation has a local deadline. Cancellation must not leave a presence owner. Optional original platform states remain deferred. |
| R5 | Load or register the account. New accounts use right 1. Ignore last-login and last-logout update errors. Required account load/register failure sends D11 reason 16. Required character/service Store load failure sends reason 16 (step-8 caller check). Restore kind-6 incomplete services into the session cache. Reject other nonempty kinds with reason 2. Do not complete or consume recovered services. The checked fresh-login loader path sends D11/16. ❓ Other DB callers remain open. |
| R6 | Send D10 only after required loads succeed. Use the checked fragment format. Optional charge/EXP/grade/schedule rows and redirect data use explicit empty/zero local values. Cache character rows. D35 reads the cache, not the Store. Send D36 with the checked fragment format. Missing session sends D37 reason 4; Init state sends reason 5; local invalid cache data sends reason 2. |
| R7 | D18 uses link plus gateKey. D19/D23 use cliKey. Local ownership checks prevent cross-link access. Close and logout remove indexes, release presence and attempt last-logout update. D23 sends D21 in Lobby state. D18/D19 do not send D21. INIT close sends no D21. Repeated close is a no-op. |
| R8 | Encode D20 with gateKey, D21 with cliKey and D22 with cliKey plus signed reason byte. Kick emits D22 for Lobby state. The checked original INIT branch closes the anonymous binding with D20. Both close the session once. Local anonymous close emits D20 with the gateKey and removes an Init session once. It does not close a Lobby session. Apply the checked D22 reason conversion: preserve 1..8, map 10..12 to 9..11, use 0 otherwise. ❓ World-kick integration and original numeric deadlines remain open. No recovered expiry duration is assumed. |
| R9 | Link loss and server shutdown release all owned sessions. Bound connection count, session count and output queues. Each link has one ordered job lane and output queue. NP waits do not hold a cross-link job lock. All lanes share key allocation, session capacity, Store and NP presence ownership. Short synchronous Store calls remain serialized. |

The binary binds loopback only. Its empty NP stub rejects all tokens.
Tests install generated fixtures through the library. No configuration sample is read.
The default NP deadline is five seconds. This is a local policy, not a recovered duration.
R10: An optional `init_timeout` expires Init and RetainedInit sessions and sends D20 once. Default: disabled, because the original duration remains open. The local scheduler checks deadlines every 10 ms and before packet dispatch. NP start holds its own lane; successful start removes the Init deadline. No local Lobby idle expiry is assumed.

R11: Generated NP fixtures can set monotonic `token_expiry` deadlines. Expiry and consumption reject a token at or after its deadline. Consumed/expired tokens cannot replay. No original NP lifetime is claimed. R13 defines local retained lobby-token issuance and expiry. Original duration bindings remain deferred. R14 adds local D24 world-token support.

R12: Restore kind-6 service rows into the session cache. Do not change persistent rows. Unknown or detail-dependent service kinds fail with reason 2.

R13: D12/D15 support same-gate-link retained sessions. The local `retain` control needs an explicit positive lifetime. A token is single-use and belongs to one cliKey and IPv4. Wrong fields do not consume it. Expiry closes the retained session once. D15 uses cached data, not new NP consumption or DB loads. Optional NP refresh is disabled. No real world-return listener or cross-gate-link transfer is included.

R14: D24 needs an explicit positive `world_token_lifetime`, generated internal eligibility, and an explicit NP world decision. Unknown metadata and NP answers reject. The GameDaemon trait has cancellation-safe restore/admit calls and an owned admission guard. The local world timeout is five seconds; it is not a recovered value. Restoration requires an explicit Store fixture and commits once before later NP checks. Any pending service blocks entry in this milestone. D25 uses an unverified local zero trailing short. D27 fields are opaque generated fixture data. Queueing emits D26/107 then D27. Queue promotion is an explicit local control. Queued and ready sessions expire at the configured token deadline and release admission. World-ready D19 is absorbed. Token consumption verifies identity before consumption, then checks world. A wrong world consumes the token, releases admission and restores Lobby under a local failure-hook policy. Success releases local admission and enters local World state. No real enter-world job or world listener is claimed.

No DB driver, platform transport, licence or anti-cheat adapter is included.

## Real-client lobby entry fixes (2026-11-08)

A real 381 client (Client.exe 381.8884) logged in through STS and LobbyGate. It then stayed on "Connecting". The LobbyGate debug log showed this order: C106, C3 (login), C24 (world list), C56/C65/C59 relayed as D64/D73/D67, C103, then only C9 pings. The client never sent C27 (characters).

| Gap | Fix | Source |
| --- | --- | --- |
| HelloOk (D2) had an empty world list. | `protocol::WorldInfo`, `Settings::worlds`, `lobbyd-sts --world <id>,<cluster>,<name>,<ip>,<port>`. Default stays empty. | Row layout: LobbyGate spec 05 §2.2. ❓ worldType/playType 0 and the flag values are local choices. The original builds rows from `config/LobbyDaemon/config/gamed-list.config.xml` and from live GameDaemons. |
| D64/D67/D70/D73/D76 (goods) had no handler and no reply. | `Jobs::goods_unavailable`: Fail reply with reason 7 (account inventory not available), 4 (no session), 5 (not in lobby). Malformed requests are dropped. | Build 307: the jobs log "init fail; account-inventory N/A" and send reason 7. GetGoodsCount 0x1400C3550/0x14004FB70 → D66 `hqE`; ListGoods 0x1400C37E0/0x140053C40 → D69 `hqE`; TakeGoods 0x140079130 → D72 `hqE` (sender 0x1400BA2B0); ListLobbyServiceItemInfo 0x1400C3D40/0x1400548A0 → D75 `hqQE` (sender 0x1400BA610, `mov r9d, 7` at 0x140054AB7); GetLobbyServiceItemInfo 0x140050780 → D78 `hqqE` (sender 0x1400BA830). The 2020 config has `use-account-inventory = no`. |

❓ Build 307 accepts client states 0x20000 and 0x10003 for these jobs. Only `State::Lobby` is mapped.

Still without a handler (the client did not send them in this test): D28, D31, D57, D60, D79, D82–D100, D106. The 307 inventory jobs for D82–D100 also check account-inventory N/A; AddSlot uses reason 234, ChangeUserName 219, ExtendJumpingCharacterCreationQuota 7.

LobbyGate: `LOBBYGATE_DEBUG=1` sets the log level to DEBUG and logs each client and lobbyd command number.
