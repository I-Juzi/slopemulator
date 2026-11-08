# 10 — Session receive handlers, build 381.8867

## Scope and notation

This file extends docs 04–07. It does not replace their limits.
This is a static specification. No client test ran.
`done` means that all game-logic paths for this receive operation were read.
It does not mean that a Rust port or a client test passed.
`partial` means that the listed paths still need research.
Build 348.8349 supplies names only. Build 381 supplies behavior and command IDs.
Addresses without a build label refer to build 381.
Full formats include command `h`. They exclude the outer size field.
Scalars use little-endian order. `h/E/u` are 2 bytes. `i` is 4 bytes.
`q` is 8 bytes. `c/Q` are 1 byte. Preserve raw bits where the sign is unknown.
`b/K` are nonnegative signed-16 byte counts followed by bytes.
`s` is counted UTF-16LE. `C` is a 2-byte repeat count, not a 1-byte scalar. Shared reader `0x14069DA00` consumes two bytes and returns their raw ushort bits. Callers can interpret the result as signed. A missing count returns low-16 `0xFFFF`. Semicolon ends a repeated group; it adds no wire byte. See docs 04–05 for other codec limits.
A handler that does not decode a body does not prove an exact-length check.
Do not add a trailing-byte rejection to such a handler.
PC routes require the dispatcher to resolve an authenticated PC.
Any-state routes do not have that requirement. See doc 09 for dispatch checks.
A reference release is not a socket close.
Use doc 07, part 4 for the conditional user-manager kick boundary.
Do not apply its BoardGacha kick rule to these handlers.
No runtime identity, address, password or key value is included here.

## 16 — G_PingReqCmd — done

- **381:** `0x140370210`. **348:** `HandleGame_PingReq`, `0x14036DF20`.
- **Wire:** `h` in the existing client constructor. The server does not decode the body.
- **Checks:** Any-state dispatch. No handler state, range, ownership, cooldown or table check.
- **State:** Write a debug log with the session ID. Return true.
- **Output/failure:** No reply, broadcast or kick. No daemon or DB call.

## 19 — G_GameKeyAuthReqCmd — done

- **381:** `0x14001AFE0`. **348 name:** `-`.
- **Wire:** No format decoder. Body bytes are unused. The command name does not prove an authentication body.
- **Checks:** Unauthenticated dispatch only. No checks in the target.
- **State/output:** Return true. No authentication change, reply, broadcast, failure, kick, daemon call or DB call.

## 22 — G_NPTokenAuthReqCmd — done

- **381:** `0x14001AFE0`. **348 name:** `-`.
- **Wire:** No format decoder. Body bytes are unused.
- **Checks:** Unauthenticated dispatch only. No checks in the target.
- **State/output:** Return true. No token processing, state change, reply, broadcast, failure, kick, daemon call or DB call.

## 34 — G_QueryWaitingNumberReqCmd — done

- **381:** `0x14001AFE0`. **348 name:** `-`.
- **Wire:** No format decoder. Body bytes are unused.
- **Checks:** PC dispatch only. No checks in the target.
- **State/output:** Return true. No queue lookup, state change, reply, broadcast, failure, kick, daemon call or DB call.

## 38 — G_LeaveWorldReqCmd — done

- **381:** `0x14001AFE0`. **348 name:** `-`.
- **Wire:** No format decoder. Body bytes are unused.
- **Checks:** PC dispatch only. No checks in the target.
- **State/output:** Return true. It does not leave the world. No reply, broadcast, failure, kick, daemon call or DB call.

## 307 — G_ClientConfigReqCmd — done

- **381:** `0x14001AFE0`. **348 name:** `-`.
- **Wire:** No format decoder. Body bytes are unused.
- **Checks:** PC dispatch only. No checks in the target.
- **State/output:** Return true. No configuration update, reply, broadcast, failure, kick, daemon call or DB call.

## 414 — G_CheckRoundTripTimeReqCmd — done

- **381:** `0x14039CCD0`. **348:** `HandleGame_CheckRoundTripTimeReq`, `0x14039C8E0`.
- **Wire:** `hqq`: command, opaque data1, opaque data2.
- **Checks:** PC dispatch. Decode through `0x14069F410`. No other checks.
- **State:** No game-state change.
- **Output:** On decode success, send 415, `hqq`, to the requesting PC. Echo both q values unchanged.
- **Failure:** On decode failure, log only. Return true on both paths. No kick or broadcast.
- **Dependencies:** No table, daemon or DB call.

## 416 — G_WriteLatencyLogReqCmd — done

- **381:** `0x14039CD70`. **348:** `HandleGame_WriteLatencyLogReq`, `0x14039C980`.
- **Wire:** `hQiiiii`: command, type byte, min, max, average, count, client-address bits.
- **Checks:** PC dispatch, then decode. No numeric bounds or table lookup.
- **State:** Log only. Types 0/1/2/3 select skill latency, skill latency overflow, RTT latency, RTT latency overflow. Other types select unknown.
- **Privacy branch:** If the daemon and config exist and config +1628 is false, replace the client-address value with zero for logging. The 348 field name is `_allowClientIPAddressInLogs`. The log also obtains server-side identity and connection metadata. No such values are part of this spec.
- **Output/failure:** No reply or broadcast. Decode failure logs only. Return true. No kick, daemon request or DB procedure.

## 11 — G_HelloReqCmd — partial

- **381:** `0x1403703D0`. **348:** `HandleGame_HelloReq`, `0x14036E100`.
- **Wire:** `hhqsqb`: command, protocol version, property version, publisher string, publisher-property version, opaque handshake bytes. Publisher decode capacity is 40 UTF-16 units. Handshake decode capacity is 4000 bytes.
- **Checks, in order:** Unauthenticated dispatch; decode; protocol equals 1278 and property version equals `0x12D261D120403`; publisher equals `PUBLISHER_CODES[daemon publisherCode]`; publisher-code switch; positive blob length; handshake-parameter operation.
- **Publisher comparison:** The UTF-16 comparison in `0x1403703D0` stops at the first NUL, not at the counted string end. The Rust decoder still consumes the complete counted string before it reads later fields.
- **Publisher switch:** Codes 1, 9, 11–15 do not check the extra version. Codes 2–8 and 10 require `0x40000000000`. Other codes fail. These are version constants, not keys.
- **State/output:** Compose 12, `hb`, with the opaque reply. Use the RSA-family handshake → AES-128 session step from doc 04. Send under the session lock. If session +256 is 3, set it to 5; release the old handshake processor at +272; clear +272; assign current processor +264 from session processor +280. The reply uses the old current processor before this assignment. Instructions in `0x1403703D0` and the 348 name guide agree.
- **Failure:** Return false after decode, version, publisher, empty-blob or operation failure. No explicit failure reply or user-manager kick in this function. ❓ The dispatcher/session consequence of false is not closed here.
- **Dependencies:** `0x1406A65D0` is the handshake-operation boundary. No DB or other-daemon call. Do not inspect key setup at `0x1406E3100`.
- **Rust boundary:** `Services::hello` supplies Unknown, Rejected or an opaque reply. Unknown never approves the operation. `Outcome::session_effects` records send-with-current-processor before session-processor activation. The adapter must apply this vector in order; these sends are not duplicated in `replies`. A known mode 3 becomes 5. The native 4000-byte packet leaves 3994 bytes for the reply after size, command and blob count. An invalid adapter length remains unsupported. No key setup or framing acceptance is implemented.
- **❓ Missing:** Full framing/processor acceptance remains outside this handler spec. No secret material is specified.

## 13 — G_MobileHelloReqCmd — done

- **381:** `0x140370890`. **348:** `HandleGame_MobileHelloReq`, `0x14036E510`.
- **Wire:** Repeat descriptor `hhsChh;`: command, protocol:h, publisher:s, count:C, repeated property-id:h and property-version:h. The repeat decoder starts after the command. Publisher capacity is 40 UTF-16 units.
- **Checks, in order:** Unauthenticated dispatch; decode initial two fields; protocol equals 1278; decode nonnegative repeat count; decode each pair; compare each pair to the switch below. The decoded publisher is not compared in this handler. Zero pairs succeed. No complete-set or uniqueness check appears.

| Property ID | Required version | 348 name guide |
| ---: | ---: | --- |
| 2 | 301 | CreatureProperty |
| 3 | 38 | WeaponProperty |
| 4 | 29 | CostumeProperty |
| 5 | 18 | GroceryProperty |
| 7 | 9 | QuestProperty |
| 8 | 9 | QuestList |
| 9 | 4 | EffectProperty |
| 10 | 3 | RecycleTimeProperty |
| 11 | 20 | GemProperty |
| 12 | 2 | DiscoveryArea |
| 13 | 2 | ActivatedTeleport |
| 14 | 4 | ClientOption |
| 15 | 20 | AccessoryProperty |
| 16, 17 | 4 | SummonedProperty, ExtraCreatureProperty |
| 20 | 3 | ContentQuotaProperty |
| 21–25 | 11 | AcquiredSkill1–5 |
| 26 | 7 | SkillPage |
| 27 | 9 | RandomStore; ID differs from 348 |
| 34, 35 | 1 | SkillSkinProperty, MentoringProperty |

- **Failure:** Send 15, `hh`, reason 2 for decode/count failure. Use reason 63 for protocol mismatch, unknown property ID or property-version mismatch. Return true on every ordinary branch. No direct kick.
- **Output/state:** Success sends command 14, `h`. Literal `0x1408CFBE8` is verified. No persistent state change, DB call, daemon call or broadcast in this function.
- **Repeat boundary:** Shared count reader `0x14069DA00` consumes two bytes. This handler converts it to signed short and rejects negative values. Thus the count range is 0..32767; missing count also takes reason 2. Pair decode failure takes reason 2. No numeric table lookup occurs; the property switch is a code constant list. No unread game-logic continuation remains.

## 18 — G_ServerPingReqCmd — partial

- **381:** `0x140370F10`. **348:** `HandleGame_ServerPingReq`, `0x14036EBC0`.
- **Wire:** `hqq`: command, sequence, timestamp. See doc 06 for timer behavior.
- **Checks/state:** PC dispatch; copy sent pair from PC +20712 for logging; decode. On success, overwrite received pair at +20728. This handler does not compare the echo against the sent pair before storage.
- **Failure/output:** Decode failure logs and returns false. Success returns true. No ordinary reply, broadcast, table lookup, DB request or daemon request.
- **STOP:** The conditional GameGuard call at `0x1403710A5` targets `0x14036DE10`. Do not follow it. ❓ False-return close consequence remains open. Timer policy remains partial in doc 06.

## 23 — G_LobbyTokenAuthReqCmd — partial

- **381:** `0x140371170`. **348:** `HandleGame_LobbyTokenAuthReq`, `0x14036EDE0`.
- **Wire:** `hhqq`: command, world ID, client key, lobby token. Decode return is not checked. Do not invent a native malformed-input result.
- **Checks, in order:** Reject arena config (`+237 != 0 && +248 != 0`) with 25/5. Decode. Require world ID >=1 and membership in daemon world list at +86748, count +86848; else 25/1937. Require nonzero client key; else 25/1939.
- **State/calls:** Build `LobbyTokenAuthReqParam` with accepted connection metadata. Construct `LobbyTokenAuthTran` at `0x14014C7B0`. Set transaction ID +24. Register through `0x1400D4FE0`, then dispatch through `0x1400D4A80`.
- **Failure/output:** Construction/registration failure sends 25, `hh`, reason 2. All visible paths return true. No direct kick or broadcast.
- **Transaction start:** Vtable `0x1409CE8B8` routes work to `0x14014CCB0`. Initialization `0x14014EE00` requires event 8 and a nonnull parameter. It copies world ID, client key, lobby token and accepted connection metadata into the transaction. It creates a pending user indexed by transaction ID. A duplicate pending-user index takes the 25 failure path. No authentication success occurs at registration.
- **Daemon request, verified in 381:** Work state 1/event 0 sends Lobby command 5, `hqqqi`: transaction ID:q, client key:q, lobby token:q, accepted connection-address bits:i. Route by the requested world ID. A send failure sends client 25/2 and removes the pending-user entry. No key values or connection values are recorded here.
- **Consume-token failure:** State 1/event 28 reads the backend reason short. Map 15/16→1939, 17→1936, 18/19/20→2, all others→1. Remove the pending-user entry, send client 25 `hh`, then transit to state 17. Source: `0x14014CCB0`, state-1 switch. This mapping does not apply to later transaction states.
- **Consume-token success subset:** State 1/event 27 calls `0x14014EFE0`. Require nonzero returned character ID and returned mode byte in 1..3. Copy account, character, charge/recovery and grade state into the pending user/transaction. Invalid response uses Lobby command 12 `hqq` to undo the consumed token before the client failure path. Later recovery, charge and DB paths remain incomplete.
- **Command 24, complete body:** Work state 16/event 0 sends `hQiQih`, then transits to state 17. Body order: charge-apply-type byte from transaction +1980; remaining-play-time:i from +1984; charge-base-type byte from +1988; charge-item-ID:i from +1992; reservations:h from +1996. Instructions `0x14014ED65..0x14014EDA7` verify all arguments. The 348 name guide is `LobbyTokenAuthTran::successLobbyTokenAuth`, `0x140156A40`; its command number is not used. The optional time-based-play branch runs before this send. Do not send 24 directly after Lobby command 5.
- **Rust subset:** The wrapper retains typed world/key/token fields only after queue success. `LobbyTokenAuthTransaction::registered` requires completed initialization and pending-user indexing. Its start emits Lobby 5. Its send callback and state-1 consume-failure callback preserve pending-user removal and termination plans. Unknown or out-of-order callbacks remain unsupported. The separate command-24 codec does not bypass the later transaction states and does not set authentication.
- **❓ Missing:** Later transaction callbacks, DB loading/procedures, all recovery/charge failures, complete rollback and conditional kick paths. Command 24 is no longer a body gap. No key material is specified.

## 26 — G_ReturnToLobbyReqCmd — partial

- **381:** `0x140371500`. **348:** `HandleGame_ReturnToLobbyReq`, `0x14036F0F0`.
- **Wire:** No request-body decoder.
- **Checks/order:** PC dispatch; reject arena config (+237 and +248); acquire PC user +20696 only if present and +20752 is zero; call `UserManager::LogoutUser` at `0x1403F0340`; construct `ReturnToLobbyTran`; register and start the transaction.
- **State:** Logout helper can change user-manager state before transaction registration. Transaction vtable is `0x140C42E80`. Its ID is stored at +24.
- **Failure/output:** Send 28, `hh`: arena reason 5, logout failure 1954, transaction start failure 2. The omitted arguments are verified in instructions. Return true. No direct broadcast or universal kick rule.
- **❓ Missing:** Logout helper, transaction body, success reply, daemon commands, DB procedures, rollback and final close path. See doc 07 part 4 for the separate kick boundary.

## 29 — G_CleanUserConnectionInfoInCligateReqCmd — done

- **381:** `0x1403717A0`. **348:** `HandleGame_CleanUserConnectionInfoInCligateReq`, `0x14036F360`.
- **Wire:** No body decoder.
- **Checks/order:** PC dispatch. If arena config +237 and +248 are nonzero, log and return true without sending.
- **Daemon call:** Otherwise call `LobbyDaemonChannelManager::Send` (`0x140149740`) for PC world ID at +96. Send command 17, `hqQQ`: PC DB ID from +20744, zero byte, one byte. Exact format bytes at `0x14099C248` are verified.
- **State/output:** No local game-state mutation. Log send success or failure. No client reply, broadcast, DB procedure or kick. Return true. No daemon-response wait is installed here.

## 31 — G_EnterWorldReqCmd — partial

- **381:** `0x1403718A0`. **348:** `HandleGame_EnterWorldReq`, `0x14036F470`.
- **Wire:** `hic`: command, clientVersion:i, characterId:c. The 348 typed function resolves the misleading log noted in doc 05: the integer is `_clientVersion`; the byte is named `characterId`. The byte is logged but is not used to select a character in the ordinary code read here.
- **Checks/order:** PC dispatch; reject arena config or PC arena state 2, 3, 5 or 6 under PC lock +37480; decode; reject `_receivedEnterWorldReq` at +12576.
- **State:** Set +12576 to true and store clientVersion at +20704. These changes precede all later failures.
- **STOP:** The optional integration virtual call at `0x140371AC8` is named `SecurityAddClient` in 348. Do not follow this path or its integration continuation. The ordinary path resumes at composite send `0x1405F5980`.
- **Ordinary continuation:** Send initial PC composite through `0x1405F5980` before destination resolution. Copy PC position +268/+270/+272 and yaw +274. If game-packet-right +20672 has bit 8, save geo-zone +264 and position at +38728..+38736. Only this branch requires MobileZone at daemon +5888. Set the target runtime zone and geo-zone from MobileZone. Read mobile spawn index 1, if available; use its pose only when record kind is 1. Missing/wrong-kind spawn records retain the copied PC pose; they do not call the translator. The branch without bit 8 calls `Pc::TranslateZoneOnEnterWorld` at `0x1405F1230`. Then call `World::Enter` at `0x1403F4A60`.
- **World entry subset `0x1403F4A60`:** A null PC fails. A zero zone ID uses the World default. Resolve the runtime zone through `0x1403F6A20`; missing zone fails. Return the spawn-queue result from `0x14022EC20`. Optional party/guild work follows that call. Its task-allocation failure does not replace the saved result. A successful queue is not a completed spawn.
- **Output/failure:** Success composite is 32; see doc 05 for its partial layout. Failure is 33, `hh`: arena 5; decode/duplicate 2; missing mobile zone 2; destination translation 1; World::Enter failure 1. Translation and World::Enter failure return false. Other handled branches return true. The omitted reason arguments are verified in instructions. No direct user-manager kick is shown.
- **❓ Missing:** Complete command 32, destination translation, world-entry callees, table-field mapping, DB source and failure cleanup. Do not reset +12576 on failure without evidence.

## 42 — G_EnterZoneReqCmd — partial

- **381:** `0x140371EE0`. **348:** `HandleGame_EnterZoneReq`, `0x14036FAA0`.
- **Wire:** `hiuuuh`: command, spawnActionKey:i, summoned x/y/z shorts, summoned yaw:h. Decode return is not checked. See doc 05 for the field correspondence.
- **Checks/order:** PC dispatch; snapshot zone +2224; decode; require zone; require user +20696 with +20752 clear. Missing user logs only. Missing zone sends 44, `hE`, reason 2.
- **State/calls:** Allocate `EnterZoneTask` with vtable `0x140BC1628`. Attach zone through `0x14025DB30`. Store caster ID +128. Add task as PC child through `0x14069BF60`. Set child flag +138. Copy correlation to +144 and summon pose to +148..+154. Queue through `0x140254FD0`.
- **STOP:** The conditional anti-bot delay call at `0x1403721D5` targets `0x1402550E0`. Do not follow that restricted branch. Zone-entry continuation also stops at anti-cheat reference `0x1405DF767` in `0x1405DE040`.
- **Task `0x1402912F0`:** A caster kind other than 1 logs only. Resolve the current spawn action through PC action-manager virtual +88 with argument zero. Missing action sends 44/2. Action state other than 2 sends 44/1149. Only a correlation mismatch at action +208 sends 44/1150. On match, call `0x1404BF580` with the summoned pose, then call `0x1405F1FA0`, unlink the task child and release it. Replies use `hE`. Do not combine the three failure cases.
- **Spawn subset `0x1404BF1E0`:** Set properties 99, 100 and 101 from the supplied position in that order. A failed set ends the action and returns false. Raise the local Z to the supplied floor when necessary. A nonzero spawn context writes creature +2328. Call `0x1402304A0` before the later `0x14026A180` update. A failed later update ends the action; it does not retract a prior send. Yaw uses property-102 bounds from metadata. Do not treat command 43 as proof that every later mutation succeeded.
- **Summon subset `0x1404BF580`:** PC spawn runs first. For a live summon and PC job 6, test the client summon position against terrain, environment availability and the squared configured distance at rule config +188. On rejection, use the PC spawn pose. Raise summon Z to the floor. Pass the client summon yaw through the property bounds. Successful summon spawn sets control state +6984 to 1. Other summon branches and cleanup remain partial.
- **Failure/output:** Queue failure sends 44, `hE`, reason 2, verified in instructions. No direct kick. No DB/daemon call in the wrapper.
- **❓ Missing:** All EnterZoneTask paths, summon correction, completion/appearance packets and downstream DB/daemon effects.

## 929 — G_ContentsRuleListReqCmd — partial

- **381:** `0x140370250`. **348:** `HandleGame_ContentsRuleListReq`, `0x14036DF60`.
- **Wire:** No request-body decoder. Do not infer an exact-length check.
- **Checks:** Any-state dispatch. Read last request tick `L` from session +200 extra-data pointer, or use zero. Compute `T = GetTickCount64() - daemon start tick`.
- **Order:** Allow if `L <= 0`, `L <= ConfigLastReloadTime`, `T <= 30000`, or `T-L >= 30000`. These are signed 64-bit comparisons in `0x140370250`. Preserve the raw tick bits. A future positive L does not pass through unsigned subtraction. Otherwise reject.
- **State:** On allow, copy the cached packet at `0x140EE5888`, if present. Store T through the extra-data pointer, if present. Apply the existing session encryption step to the copied buffer. Queue it under the session lock through `0x1406A6420`. Clear the temporary packet. No cryptographic internals are specified.
- **Failure:** Send 931, `hE`, reason 2167 to this session. No direct kick. Return true.
- **Dependencies:** No DB or other-daemon call in the receive handler. Reload-time global: `0x140EE5898`.
- **Cache owner:** Initial load `0x140087330` reads 167 rule definitions and calls builder `0x140087D40`. Reload `0x140087720` reads 18 reloadable definitions, clones the config, increments revision `0x140EE586C`, then calls the builder. Success atomically replaces cache/config and stores daemon-relative reload tick in `0x140EE5898`. Failure decrements the revision and keeps the previous cache/config. Old objects enter deferred-release lists.
- **Success builder `0x140087D40`:** Creates command 930 with fragment handling. It converts an XML header, supported option entries and the closing config element to UTF-8. Each entry has a counted byte block. Types 1/2/3/4/6 select formatting branches. The loop covers 167 definitions. Do not copy the build-348 command ID.
- **Tick prefix:** The wrapper writes T before encryption/send, including the null-cache branch. A missing extra-data pointer means no tick storage. The Rust plan records this verified mutation and still stops before the incomplete send contract.
- **❓ Missing:** Exact build-381 fragment prefix/metadata, config option field mapping and null-cache send behavior. Do not substitute an empty success packet.

## 174 — G_ClearLogoutPartyMemberReqCmd — partial

- **381:** `0x14038DF10`. **348:** `HandleGame_ClearLogoutPartyMemberReq`, `0x14038D6A0`.
- **Wire:** No body decoder.
- **Checks/order:** Require PC zone +2224. Construct task through `0x14036D800`. Initialize through `0x1402B4920(zone, PC)`. Queue through `0x140254FD0`. Each failure sends 175, `hh`, reason 2. Return true.
- **State:** Task gets zone/caster and child relation. No direct party mutation in wrapper.
- **Task:** `ClearLogoutPartyMemberTask`, vtable `0x140BBA228`, start `0x1402B49D0`. Require nonzero party ID at PC +21112, else send 175/2. With a party, call `TeamManager::ClearExpiredPartyMember`, `0x1401DD540`. The 348 name guide is start `0x1402B7F90`.
- **Manager subset:** Acquire manager then team lock. Resolve team by runtime PC ID; failure sends 175/2. If team byte +124 is set, require leader account/object pair +112/+104 to match PC +20744/+40; mismatch sends 263/2. Otherwise require helper `0x1401EE320` to accept account/object and mode 3; failure sends 175/2. Iterate party indexes in team +88, count +52. Resolve each party through `0x1401EE540`; skip parties with nonzero challenge link +176. Gather expired-member results through `0x1401EE620`. Dispatch result-specific removal helpers before notification `0x1401EF4D0`, which runs after manager unlock while team lock remains held. No generic success packet is proved.
- **Output/calls:** No immediate success or broadcast. No direct DB/daemon call or kick.
- **❓ Missing:** Exact leader-state meaning of team +124, membership-mode 3 rule, expiration predicate, result mutation helpers, notification bodies/recipients, DB/daemon work and all late failures.

## 262 — G_ClearLogoutTeamMemberReqCmd — partial

- **381:** `0x140393200`. **348:** `HandleGame_ClearLogoutTeamMemberReq`, `0x1403931A0`.
- **Wire:** No body decoder.
- **Checks/order:** Require config +725 (`_teamSystemEnable` in 348). Require zone +2224. Construct and initialize the same task as command 174. Queue through `0x140254FD0`.
- **Failure/output:** Any failed wrapper check sends 263, `hh`, reason 2. Return true. No direct kick, success or broadcast.
- **State/dependencies:** Same queued task as 174. No direct team mutation or DB/daemon request.
- **Deferred output:** Shared task `0x1402B49D0` still sends 175/2 when party ID is zero, even for request 262. The manager also has both 175/2 and 263/2 branches. Preserve this distinction.
- **❓ Missing:** Same remaining manager, mutation and notification paths as 174.

## 300 — G_ChannelListReqCmd — partial

- **381:** `0x140394150`. **348:** `HandleGame_ChannelListReq`, `0x140394200`.
- **Wire:** `h`. Decode return is ignored.
- **STOP:** Restriction helper call at `0x140394181` targets `0x14060E280`. The 348 name identifies an anti-bot restriction query. Do not follow it.
- **Ordinary checks/state:** After the permitted branch, require zone. Allocate ChannelListTask, vtable `0x140B95958`. Require native PC kind +46 equal to 1. Attach zone/caster and child relation. Store geo-zone ID at task +144. Queue. Wrapper failure sends 302, `hh`, reason 2. Return true.
- **Task `0x140296990`:** Under World ZoneMgrCS at daemon +5896, find current geo-zone in PersistentZoneList at +5864. Copy `maxChannelCount` from channel-list +0 and 51 enable bytes from +3208..+3258. Unlock before send. Missing entry sends 302/1189. Found entry sends 301, `hhb`: maximum channel count, counted 51-byte enable block, to requester only.
- **State/calls:** No list mutation, broadcast, DB or daemon request in the ordinary task. No direct kick.
- **❓ Missing:** Restricted branch remains excluded. Channel-list population and the meaning of each enable byte remain open. Do not synthesize list entries from table count.

## 303 — G_ChangeChannelReqCmd — partial

- **381:** `0x1403943A0`. **348:** `HandleGame_ChangeChannelReq`, `0x140394480`.
- **Wire:** `hh`: command, signed channel short. Decode return is ignored.
- **Wrapper checks:** Require zone; restriction-query boundary at `0x140394427` → `0x14060E280` (STOP); ordinary branch creates ChangeChannelTask, vtable `0x140BA7EA8`; require PC kind 1; store channel at task +144; attach/queue. Missing zone, wrong kind or queue failure sends 304/2. Restriction-result failure sends 304/1196. Replies use `hh`. Return true.
- **Task start:** `0x140296BF0`. All failures below send 304, `hh`, to requester.

| Order | Check | Reason on failure |
| ---: | --- | ---: |
| 1 | No achievement-reward flag +21096 and no jumping-ticket flag +13752, each under its lock | 1194 |
| 2 | PC +888 bit 0 is clear (combat mode in 348) | 2 |
| 3 | Trying-instant-payment flag +21080 is clear under its lock | 1195 |
| 4 | Outside arena daemon, PC arena session is not 2, 3, 5 or 6 | 5 |
| 5 | Requested channel differs from zone +16600 | 1190 |
| 6 | For positive channel, `World::AvailableChannel` accepts current geo-zone/channel | 1192 |
| 7 | If builder_right +532 <=0 and ignore-cooldown +12580 is zero: last change tick +20872 is zero or elapsed >=160000 | 1193 |
| 8 | For non-builder: geo-zone is not either configured destination (+632/+636 in rule config), not zone type 40, and not type 87 while rule config +932 is enabled | 1191 |
| 9 | If zone attraction +16536 exists, its virtual +296 channel check succeeds | Supplied short |
| 10 | Without attraction, requested channel is not zero | 1191 |
| 11 | Action class 3 can be created and initialized through `0x1404C2920` | 2 |

- **State:** On successful initialization only, store daemon-relative tick at PC +20872. Post LeaveAction through `0x1404C3090`. This precedes deferred completion.
- **STOP:** Task's availability check also calls restriction helper `0x14060E280`. Do not follow that branch.
- **❓ Missing:** AvailableChannel implementation, attraction virtual variants, zone type field name, LeaveAction routing/rollback, success packets, broadcasts, daemon commands and DB procedures. Negative channels skip the positive-channel availability check; do not add a blanket positive-only rule.

## 305 — G_SaveClientOptionReqCmd — partial

- **381:** `0x140394600`. **348:** `HandleGame_SaveClientOptionReq`, `0x140394750`.
- **Wire:** `hQb`: command, option type byte, counted option bytes. Decode destination capacity is 4000.
- **Checks/order:** Decode; require signed length <=3300; require option type 1 or 2. Failures send 306, `hh`, reason 2. Return true.
- **Daemon helper:** `CachedChannel::SendClientOptionData`, `0x140317380`. If PC DB ID +20744 is zero, log only. Otherwise compose exact repeat descriptor `hQCqqchb;` at `0x140B69EA8`. The instruction sequence at `0x140317492` starts command 17, then byte zero, then a repeated record. Each record starts with PC DB ID:q, sign-extended option type:q, property ID 14:c and version 2:h, followed by the counted block. The repeat count occupies two bytes. One record is added before the helper sends the packet. The helper embeds version short 2, original length short and raw option bytes in a counted block. It sends through `0x14032AB10`.
- **State/output:** No local option assignment in wrapper. No immediate success or broadcast. No direct kick.
- **❓ Missing:** Cached response handler, DB procedure, persistence failures and success reply. Do not treat queueing as a completed save.

## 411 — G_AutoLogoutCheckReqCmd — done

- **381:** `0x14039CB30`. **348:** `HandleGame_AutoLogoutCheckReq`, `0x14039C730`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone; create AutoLogoutCheckTask, vtable `0x140BB3598`; require PC kind 1; attach zone/caster and child relation; queue. Failed wrapper checks send 413, `hh`, reason 2. Return true.
- **Task `0x1402A0800`:** If config +537 (`_gameLogEnable` in 348) is clear, or caster is absent/not kind 1, do nothing. Otherwise derive state detail 1/2/3 for PC state +578 equal to 0/1/2, else zero. Build a report with current PC/zone metadata. Call `0x140200860`.
- **State/output:** No logout, timer change or client success appears in the read task. The name alone does not prove logout. No direct kick or broadcast.
- **Report sink:** Instructions in `0x140200860` call `GetLocalTime` and `0x1406A2FC0` at `0x140200BBA`. Report ID 1009 is a log identifier, not a client command. The sink uses optional LogArchive global `0x140F77BB8`; it composes a type-5 archive record through `0x1406C0E50` and queues it through `0x1406AE790` on archive +48. An absent archive does nothing. No game reply, logout, timer mutation or DB procedure occurs in this report path. Queueing does not prove archive delivery.
- **Rust plan:** Keep only the report ID and state detail. Omit identity, address and formatted PC metadata. This is a redacted log plan, not the native archive wire codec.

## 422 — G_CurrentUTCTimeReqCmd — done

- **381:** `0x1403A1AB0`; sender `0x1405F88C0`. **348:** `HandleGame_CurrentUTCTimeReq`, `0x14039FC00`; `Pc::SendCurrentUTCTimeOk`, `0x1405EE6F0`.
- **Wire:** No body decoder.
- **Checks/state:** PC dispatch only. Read `_time64(nullptr)`. No game-state mutation.
- **Output:** Send 423, `hq`, to requester. The q is the native signed 64-bit UTC Unix time in seconds. The command immediate `0x1A7` and saved `_time64` result are verified in sender instructions.
- **Failure/dependencies:** No error reply, broadcast, table, kick, daemon request or DB call. Handler returns true.

## 809 — G_ExtendJumpingCharacterCreationQuotaWithWarehouseItemReqCmd — partial

- **381:** `0x1403C1580`. **348:** `HandleGame_ExtendJumpingCharacterCreationQuotaWithWarehouseItemReq`, `0x1403BC2E0`.
- **Wire:** `hqqi`: command, labelId:q, itemInstanceId:q, itemDataId:i. Decode return is ignored.
- **Checks/state:** PC dispatch only. Log that the operation is unavailable. No ownership, quota, inventory or table check. No mutation.
- **Output:** Always send 811, `hEqq`, reason 1967, decoded labelId and itemInstanceId. Return true. No broadcast, kick, daemon call or DB request.
- **❓ Missing:** Native output bits on malformed input. Locals for the echoed q values are not initialized before unchecked decode. Valid-input behavior is fully read. Do not implement the operation named by the request.

## 812 — G_UpgradeJumpingCharacterStateWithCreationQuotaReqCmd — done

- **381:** `0x1403C1650`. **348:** `HandleGame_UpgradeJumpingCharacterStateWithCreationQuotaReq`, `0x1403BC3B0`.
- **Wire:** No body decoder.
- **Checks/state:** PC dispatch only. Log unavailable operation. No quota, character-state or table check. No mutation.
- **Output:** Send 814, `hE`, reason 1974 to requester. Return true. No broadcast, kick, daemon call or DB request.

## 887 — G_QQLogoutReqCmd — partial

- **381 dispatch:** `0x1400DE550`. **348 name:** `-`.
- **Thunk proof:** Bytes at dispatch start encode a relative jump to `0x140371430`. No function creation or database change was needed.
- **Wire:** No decoder in the thunk target.
- **Checks/order:** If daemon +2720 is null, return false. Otherwise get PC user +20696 only when nonzero and PC +20752 is clear. If no user, return true without work.
- **Calls/state:** With a user, call interface virtual +8 on daemon +2720 with user metadata pointer +40 and length +104. Release user and return true. No direct PC state mutation, reply, broadcast or kick.
- **❓ Missing:** Interface identity, external call effects, daemon/DB ownership, and dispatcher handling of false. Do not mark the operation unused or complete.

## 981 — G_EnterZoneDirectReqCmd — partial

- **381:** `0x1403CD3C0`. **348 name:** `-`.
- **Wire:** Exact descriptor `hQqcChh;`: command, entry-type byte, opaque q, flag byte, count:C, repeated two shorts. ❓ Semantic names for q, flag and pairs need types from a matching version.
- **Checks/order:** Reject arena config or arena session 2/3/5/6 with 983/5. Decode first three fields; failure is 983/2. Require repeat count 0..16; failure is 983/2399. Decode pairs; failure is 983/2. Require zone; failure is 983/2. Create task vtable `0x140B9D6A0`; require PC kind 1 and entry-type 1..3; attach and queue.
- **State:** Task entry type +144, q +152, flag +160, count +162, pair array +164 (4-byte stride, capacity 16). Common caster/zone/child state also changes.
- **Failure/output:** Init/queue failure sends 983/2. Replies use `hE` and go to requester. Return true. No immediate success, broadcast or kick.
- **Task `0x1402BDCA0`, ordered prefix:** Recheck arena (983/5). Require zone +16224 (983/2), then its record and record category byte +40 equal to 1 (983/2388). Require PC life-state byte +578 zero (983/2390), combat bit at +888 clear (983/2391), no achievement/jumping-ticket flags (983/2392), and no instant-payment flag (983/2393). Entry type 1 calls `0x1402BE630`; type 3 calls `0x1402BF0B0`; type 2 sends 983/3 at this stage. Nonzero helper reasons pass through.
- **Task continuation subset:** A positive cost exceeding current currency sends 983/2401. Nonempty dispose orders must pass `Inventory::CheckDispose` with reason 18; failure sends 983/2399. Resolve destination through table slot 3243. Create LeaveAction class 3, mode 3, movement reason 20. Store type-1 flag at action +54414. Resolve named return-to-game/move-to-arena records through table slot 2703. Successful initialization attaches cost/dispose orders, posts the action, then sends 982 `h`. This reply does not prove completed travel. Create/init failure sends 983/2.
- **❓ Missing:** Entry helper authorization, content/zone table fields, LeaveAction cost/disposal execution, spawn path, daemon calls and DB procedures.

## 984 — G_LeaveZoneDirectReqCmd — partial

- **381:** `0x1403CD930`. **348 name:** `-`.
- **Wire:** `h`.
- **Checks/order:** Decode; require zone; construct through `0x14036D9A0`; initialize through `0x1402BF490(zone, PC)`; queue. Initialization requires nonnull zone/caster and native kind 1. Each failure logs only. Return true.
- **State/output:** Wrapper only creates task/child state. No direct client failure or success, broadcast, DB request, daemon call or kick.
- **Task `0x1402BF540`:** If PC +13740 is nonzero and zone helper `0x140254F60` returns nonzero for the PC job/state, log only. Next, achievement/jumping-ticket flags log only; instant-payment flag logs only. Otherwise zone +17568 selects `0x140617440`. Without that pointer, a present attraction at zone +16536 receives virtual +248 with the PC. No attraction means no work. No direct client reply occurs in this task.
- **❓ Missing:** Zone helper predicate, `0x140617440`, attraction virtual variants, destination/ownership rules, persistent state and backend work.

## 1149 — G_UseJumpingCharacterTicketReqCmd — partial

- **381:** `0x1403D89E0`. **348:** `HandleGame_UseJumpingCharacterTicketReq`, `0x1403D3600`.
- **Wire:** `hqqqc`: command, itemInstanceId:q, labelId:q, third q, factionId:c. The third q writes into `itemDataId` plus the adjacent 4 bytes in the native parameter object. Only its low 32 bits are used for the wrapper table check. Preserve all eight wire bytes. ❓ Full parameter layout requires a type check.
- **Checks/order:** Decode (2483); account-inventory config +404 enabled (2); not arena config +237/+248 (5); user exists and PC +20752 clear (2); zone exists (2); low-32 itemDataId belongs to sorted jumping-ticket list at DataManager +82432 (2499); additional-service item lookup at DataManager table pointer +11640 with key `itemDataId | 0x100000000` succeeds (2499); transaction creation and registration (2).
- **Failure:** Send 1151, `hEq`, with the parenthesized reason and q zero to requester. Return true. No direct kick or broadcast.
- **State/calls:** Construct `AccountInventoryJumpingCharacter2TicketTran` at `0x1401C30B0`; assign transaction ID +24; register/start through `0x1400D4FE0`/`0x1400D4A80`. No direct item consumption or character mutation in wrapper.
- **❓ Missing:** Exact additional-service table name/fields, transaction request/response commands, item ownership/consumption, quota/character changes, DB procedure, rollback, success and conditional kick paths.

