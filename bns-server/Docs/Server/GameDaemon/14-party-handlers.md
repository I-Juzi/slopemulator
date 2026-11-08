# 14 — Party and team receive handlers, build 381.8867

## New shared helpers

| Address | Purpose | Limit |
| --- | --- | --- |
| `0x1402B2810` | Initialize the leave-party task | See command 168. |
| `0x1401D3F70` | Resolve and lock team by PC DB/runtime identity; return party/member index pair | Exact map and membership checks below. |
| `0x1401EE320`, `0x14021D070` | Resolve party/member indexes with a state filter | Exact mode-3 predicate below. |
| `0x1401E1DD0` | Change team invite authority | See command 256. |
| `0x1401F2A30` | Cancel a party-ban vote | See command 220. |
| `0x1403F7D20`, `0x1403F7EF0` | Resolve a live PC by runtime ID or name | Rights mask 6 on all calls specified here; details below. |
| `0x1402132A0` | Send a packet to online party members | Iterate party +8 records, stride 128, count +36; require record +24=3; resolve runtime ID +16; copy/send packet to each found PC. No proximity filter. |
| `0x1401EC100` | Change party invite authority | See command 182. |
| `0x140213760` | Change party pickup rule | See command 176. |
| `0x1402140D0` | Change party difficulty | See command 179. |
| `0x1401E22D0` | Difficulty result and team sync | See command 179. |

The locked resolver requires nonzero PC DB ID/runtime ID and `HIWORD(runtime ID)=1`. It locks manager +144, resolves the team through `0x1401E73E0`, locks the team, then checks `0x1401EE320(team,DB ID,runtime ID,3,0,&pair)`. Failure unlocks/releases and returns null. Success releases the manager lock but retains team +128. The pair low dword is the party index. Its high dword is the member index. Callers unlock/release the team after use. Map helper `0x1401E73E0` uses low 16 bits of PC runtime ID as an index into manager +80 qword array, bounded by manager +72 dword. The array stores team runtime IDs. Team lookup `0x1401E72D0` requires ID high word 64; derive index `(team ID & manager +24 mask) >> manager +40 shift`; require index < manager +56 dword; read pointer from manager +64, stride 16. It does not compare the full ID again in the read helper.

Membership helper `0x1401EE320` finds the PC DB ID in the ordered map at team +72. The map value gives party index. Require index >=0, index < team +48 and a nonnull pointer in team +56. `0x14021D070` scans that party's member array (+8, stride 128, count +36). Mode 3 requires member +8 DB ID equal to requester, +16 runtime ID equal to requester and +24 dword exactly 3. It returns the first matching member index. Missing map/party/member fails. No table or backend request occurs in these resolver helpers.

World runtime lookup `0x1403F7D20` requires nonzero ID and high word 1. Under World +168 lock, derive index `(ID & descriptor +24 mask) >> descriptor +40 shift`, with descriptor at World +152. Require index <20000. Fetch PC from World +136, stride 8. With rights mask 6, require `(PC +20672 ushort & 6)!=0`. A found array pointer is not compared again against the full runtime ID. Name lookup `0x1403F7EF0` uses a NUL-terminated nonempty UTF-16 name and the name map at World +144 under the same lock; it applies the same rights gate. Counted request names still consume all wire units before this NUL-terminated lookup. Name-map helper `0x140433BB0` uses CRT `wcsicmp` for tree traversal and equality. The lookup is case-insensitive. The CRT locale controls non-ASCII comparisons; do not replace it with Unicode lowercase conversion. Source: build 381.8867, `0x1403F7EF0` and `0x140433BB0`.

### Excluded call sites

| Handler | Call sites | Target | Rule |
| --- | --- | --- | --- |
| 157 | `0x14038CCA5`, `0x14038CCB1`, `0x14038CCCC`, `0x14038CCD8` | `0x14060E280` | Excluded restriction query. Stop at call. |
| 234 | `0x140390A37`, `0x140390A43`, `0x140390A8B`, `0x140390A97` | `0x14060E280` | Excluded restriction query. Stop at call. |

## Scope and notation

Use the notation and shared helpers in `10-session-handlers.md`. Use the dispatch rules in `09-parity.md`. The dispatcher is `0x14007BC00`. The authenticated-PC table is `0x1407D9F80`; each slot is `base + id*8`. Build 348 PDB hints supply names only. All behavior and addresses below come from build 381. No live test ran.

`done` means all game-logic paths were read. `partial` lists unread paths with ❓. Do not treat a queued task as a completed operation. Decode return values that the native caller ignores do not prove a malformed-input rejection. Do not add an exact-length check. Offsets are decimal unless marked `0x`.

Shared task attachment uses `0x14025DB30` (zone), task +128 (caster runtime ID), `0x14069BF60` (PC child link), and task +138=1. Queue through existing helper `0x140254FD0`. Release through `0x14000B730`; this is not a kick. See `07-server-validation.md`, part 4. No generic kick rule applies here.

## 155 — G_CreatePartyReqCmd — done

- **381:** `0x14038C9B0`. **PDB hint:** `HandleGame_CreatePartyReq`.
- **Wire:** No body decoder. Do not infer an exact-length check.
- **Checks/order:** Authenticated-PC dispatch only. Log that party creation is unavailable.
- **State:** No party creation or other mutation.
- **Output/failure:** Always send command 156, `hE`, reason 2, to self. Return true. No broadcast or kick.
- **Dependencies:** No table lookup, daemon command or DB procedure.

## 168 — G_LeavePartyReqCmd — partial

- **381:** `0x14038D940`. **PDB hint:** `HandleGame_LeavePartyReq`.
- **Wire:** No body decoder.
- **Checks/order:** PC dispatch; require zone at PC +2224 (169/764); allocate task (169/2); initialize through `0x1402B2810(task, zone, PC, 0, 0)` (169/2); queue (169/2). All wrapper replies use `hh` and go to self. Return true.
- **State:** Wrapper creates task/child state only. It does not directly remove the member.
- **Output/dependencies:** No immediate success, broadcast, kick, daemon request or DB call in the wrapper.
- **Initializer `0x1402B2810`:** Require PC kind 1; attach; task +144=0 qword and +152=0 dword for this request.
- **Task `0x1402B2920`:** Ordinary route calls `0x1401D55D0(manager,PC,0,0,1)`. With arena config and daemon +4152 byte=2, suppress this call if current PC zone subtype is 164. Also suppress it if the task zone has an attraction whose virtual +8 returns 10. Other paths call the manager. No direct task reply.
- **❓ Missing:** Manager membership/removal paths, leader replacement, replies/broadcasts and downstream backend work.

## 170 — G_DelegatePartyLeaderReqCmd — partial

- **381:** `0x14038DAA0`. **PDB hint:** `HandleGame_DelegatePartyLeaderReq`.
- **Wire:** `hq`: target runtime object ID. Decode return is ignored.
- **Checks/order:** Decode; require zone; create task, vtable `0x140BC23A0`; require native PC kind +46=1; attach; store target at task +144; queue. Each wrapper failure sends 171, `hh`, reason 2, to self. Return true.
- **State:** Wrapper changes task/child state only.
- **Output/dependencies:** No immediate success, broadcast, direct kick, table, daemon request or DB call.
- **Task `0x1402B4710`:** Resolve target through world-ID lookup with rights 6 first; missing target sends 171/779. Then require requester party ID, else 171/779. Call `0x1401E1170(PC,target)`. Release target.
- **Manager `0x1401E1170`:** Resolve locked requester team/member pair, else 171/779. Record whether requester matches both team leader DB ID +112 and runtime ID +104. Require target mode-3 membership in requester's party through `0x14021D070`. Call party-leader mutation `0x1401EB9C0`; false gives 171/779. If requester was team leader, also call team-leader transfer `0x1401F3D30`; failure gives 171/886. In team mode +124=1, successful team-leader transfer sends 272, `hq`, with current team-leader runtime ID; delegation by a non-team-leader sends 273, `hq`, with target runtime ID. Send across indexed parties through `0x1402132A0`. Unlock/release team.
- **❓ Missing:** Exact authorization and mutation inside `0x1401EB9C0` and `0x1401F3D30`, their notifications/backend effects and failure rollback. Do not assume the first mutation rolls back if the second fails. Unchecked malformed-input target bits are not specified.

## 172 — G_BanishPartyReqCmd — partial

- **381:** `0x14038DCB0`. **PDB hint:** `HandleGame_BanishPartyReq`.
- **Wire:** `hc`: target member slot byte. Decode return is ignored.
- **Checks/order:** Decode; reject arena config (+237 nonzero and +248 nonzero) with 173/782; require zone (173/2); create task, vtable `0x140BC1D08`; require PC kind 1 (173/2); attach; store byte at task +144; queue (173/2). Replies use `hh`, to self. Return true.
- **State:** Wrapper creates task/child state only. A party ban is not a connection kick.
- **Output/dependencies:** No immediate success, broadcast, direct socket kick, table, daemon request or DB call.
- **Task `0x1402B4840`:** Require PC party ID +21112 nonzero, else self 173/781 `hh`. Otherwise call `0x1401D71D0(manager,PC,unsigned slot,0)`. The byte is not a member-vector index. The manager searches member slot byte +2. Source: build 381.8867, task vtable `0x140BC1D08` +72.
- **Manager entry `0x1401D71D0`:** Lock manager +144; resolve team by requester runtime ID; lock team. For command 172, require requester mode-3 membership and both DB/runtime identities to equal party member index 0, else 173/781. Search the first member with slot byte equal to the request and state not 2; missing target also gives 781. Later paths can send 785 for party +176 nonzero or blocked target content, 784 for target PC +19640 nonzero, and 782 for an unresolved state-1 target. State 4 uses direct removal helpers; other states use `0x1401E8780`. These paths are read at the call level only. No removal helper is treated as completed work.
- **❓ Missing:** Removal-helper effects, leader/disband handling, late notifications and backend work inside `0x140218370`, `0x1401E8780`, `0x1401EA360`, `0x1401F1710`, `0x14021B6B0` and their callees. Rust closes the task's missing-party reply and stops at the manager adapter. Unchecked malformed-input byte is not specified.

## 176 — G_ChangePartyPickupRuleReqCmd — partial

- **381:** `0x14038E060`. **PDB hint:** `HandleGame_ChangePartyPickupRuleReq`.
- **Wire:** `hQcqq`: command:h, pickup-rule byte:Q, grade byte:c, designated-member runtime ID:q, money threshold:q. Instructions in `0x14038E060` verify the case-sensitive descriptor. Decode return is ignored. The initial command destination is a dummy. The first body byte must be <=3 unsigned. The wide manager log confirms auction item grade, dedicated member and auction start price; see below.
- **Checks/order:** Decode return ignored; first byte <=3; zone exists; create task, vtable `0x140B984D8`; PC kind 1; attach; queue. Each wrapper failure sends 178, `hh`, reason 2, to self. Return true.
- **Task fields:** +144 dword=sign-extended first byte; +152 qword=third decoded value; +160 byte=second byte; +168 qword=fourth decoded value.
- **State/output:** No direct rule mutation or success send in wrapper. No direct kick, table, daemon or DB call.
- **Task `0x1402B4A80`:** Require nonzero PC party ID +21112, else 178/787. Resolve the locked team/member pair through `0x1401D3F70(PC DB ID, runtime ID, &pair)`, else 178/786. Validate low-dword party index against team +48 and party vector +56. Call `0x140213760(party, high-dword member index, rule, grade, target, money)`. Invalid party index/pointer or false helper result sends 178/786. Unlock team +128 and release it. Replies use `hh`, to self.
- **Party helper `0x140213760`:** Only member index zero can change rules. Reject other indices with false. Require signed grade >0 and <= daemon rule config +508 dword. If designated ID is nonzero, scan party +8 member vector, count +36, stride 128; require a matching runtime ID at member +16 and member state +24=3. If rule=3 and daemon +2752 short is not 2, log/assert then replace rule with zero. Set party +68 dword=rule, +80 byte=grade, +72 qword=designated ID, +88 qword=money. Compose 177, `hQcqq`, from these fields. Send through `0x1402132A0`. No unchanged-value rejection. A negative money value is not rejected in this helper. Optional config +537 logs after mutation; failure to resolve the logging PC does not undo success.
- **Field names:** Wide log `0x140B49420` names the fields `pickupRule`, `auctionItemGrade`, `dedicatedMember` and `auctionStartPrice`. Config +2744 dword +508 is the maximum accepted auction item grade. Rule 3 is random-member pickup (`0x140B49200`). The final qword is an auction start price, not a threshold.
- **Optional report:** Save the old rule byte, grade byte and price before mutation. Config +537 must be set, and at least one of these three fields must change. A dedicated-member-only change does not cause a report. Resolve the leader with rights 6; missing PC skips the report without rollback. `0x140203F80` operation 1 receives the PC/zone/team snapshots, old/new rule, grade, price, member count and difficulty code (1→13, 2→11, 3→12). The sink updates a LogArchive counter and calls `0x1406A2FC0`; this is log output, not a gameplay DB request. Rust leaves this report sink as an explicit unsupported boundary when enabled.
- **Completion:** All game-logic paths are read. The shared resolver and broadcast contracts apply. No hidden gameplay backend call occurs. Sources: build 381.8867, `0x140213760`, `0x140203F80`, `0x1406A2FC0`.

## 179 — G_ChangePartyDifficultyTypeReqCmd — partial

- **381:** `0x14038E770`. **PDB hint:** `HandleGame_ChangePartyDifficultyTypeReq`.
- **Wire:** `hc`: difficulty byte. Decode return is ignored.
- **Checks/order:** Reject arena config or PC arena state +37520 in {2,3,5,6} (181/5). Decode. Require unsigned byte in 1..3 (181/2). Require zone (181/2). Create task, vtable `0x140C3ACB0`; require PC kind 1; attach; write byte to +144; queue. Kind/queue failure uses 181/2.
- **Output:** All wrapper failures use **`hc`**, not `hh`, to self. Return true. No immediate success or broadcast.
- **State/dependencies:** Task/child state only. No direct table, daemon, DB or kick.
- **Task `0x1402B51F0`:** Missing PC party ID sends 181, `hc`, from argument 789. Thus the wire reason is low byte 21. Otherwise call `0x1401E22D0`.
- **Manager `0x1401E22D0`:** Resolve locked team/pair; failure sends 181/2. Validate party index/pointer; invalid gives reason 2. Call `0x1402140D0(party,member index,difficulty,0)`. Nonzero reason sends 181, `hc`, with the low byte. On zero, send self **193 `hc` first, then 180 `hc`**, both with difficulty. On either helper result, call `0x1405FAE30` with current team/party metadata. If team +124 is nonzero, send 275, `hcc`, to all online members of all indexed parties: party index byte and current party difficulty byte. This team sync also runs after a rejected change. Unlock/release team.
- **Party helper `0x1402140D0`:** Require member index zero (790); reject unchanged difficulty (791). Resolve difficulty 1..3 through DataManager table pointer +5688 (slot 711); missing/outside gives maximum zero, otherwise signed byte record +24. Require maximum >= party member count +36, else 793. Require party +1600 and +1656 signed dwords <=0, else 794. Set party +56 byte=difficulty, then call `0x14021EDE0(party,0,0)`. Optional config +537 writes a report; missing logging PC does not change success. Return zero.
- **Capacity source:** DataManager table slot 711, record signed byte +24, gives maximum member count. ❓ No symbolic table/field name is verified. The offset and signed comparison are verified. Party +1600 counts party-zone members; +1656 counts prepare-zone members (wide log `0x140B495F0`).
- **Metadata dispatch `0x14021EDE0`:** This helper does not resize the party. Iterate members with state 3. For command 179, skip index 0. For command 259, include index 0. Resolve each runtime ID with rights 6. Missing PC, missing zone, non-PC kind or dispatch failure logs and continues. Create native kind-207 task, vtable `0x140B1DDD8`, with team runtime ID +144 and party runtime ID +152. Attach to that member and dispatch through `0x140603BD0`. Do not execute it during the change request.
- **Metadata task `0x1402B52F0`:** Missing PC party ID or locked team logs only. Require current team +40 and party +48 to equal the two queued IDs, else log only. Save old PC difficulty byte +21120. Call `0x1405FAE30` with current team mode, team/party IDs and party difficulty. If the old difficulty differs from party difficulty, send self 193 `hc` with current difficulty. No reply on an unchanged byte.
- **PC write `0x1405FAE30`:** Require PC +21104 and +21112 to equal the supplied team and party IDs. On match, write +21120=difficulty and +21136=team mode. On mismatch, log only. It does not write the IDs. The command-179 manager calls this after either helper success or rejection, before the team sync.
- **Optional report:** After metadata dispatch, config +537 and a resolved leader enable `0x140203F80` operation 2. It records the same snapshots as 176, with old/new difficulty codes. The sink is LogArchive output. Rust stops at this report adapter before command 193/180 or team success sync. Failed leader lookup skips it. No gameplay DB request occurs.
- **Completion:** All game-logic paths are read. ❓ Short unchecked decode can expose an uninitialized byte; Rust stops there. Sources: build 381.8867, `0x1402140D0`, `0x14021EDE0`, `0x1402B52F0`, `0x1405FAE30`, `0x1401E22D0`.

## 182 — G_ChangePartyInviteAuthReqCmd — done

- **381:** `0x14038EA50`. **PDB hint:** `HandleGame_ChangePartyInviteAuthReq`.
- **Wire:** `hQ`: invite-authority byte. Decode return is ignored.
- **Checks/order:** Decode; require byte <=1 unsigned (184/798); require zone (184/2); create task, vtable `0x140BC1CB0`; PC kind 1; attach; store sign-extended byte as dword +144; queue. Kind/queue failure uses 184/2.
- **Output:** Failures send 184, `hh`, to self. Return true. No immediate success or broadcast.
- **State/dependencies:** Task/child state only. No direct table, daemon, DB or kick.
- **Task `0x1402B4CC0`:** Require nonzero PC party ID +21112, else 184/796. Resolve locked team/member pair through `0x1401D3F70`, else 184/796. Call `0x1401EC100(team,&pair,authority)`. Nonzero returned short becomes 184/that reason. Unlock/release team.
- **Manager `0x1401EC100`:** Validate party index (pair low dword) against team +48 and vector +56, else reason 796. Require member index (pair high dword) zero, else 797. Reject unchanged party +64 dword with 798. Set party +64 to requested authority. Compose command 183, `hQ`, with low byte of new value. Send through `0x1402132A0(party,packet)`. Return zero. No direct table, daemon, DB or kick in this manager.
- **Completion:** The shared resolver and recipient contracts above close both gaps. All game-logic paths are read. The manager writes a diagnostic log after command 183. It does not call a backend or a report sink. Source: build 381.8867, `0x1401EC100`.

## 157 — G_InvitePartyReqCmd — partial

- **381:** `0x14038CA10`. **PDB hint:** `HandleGame_InvitePartyReq`.
- **Wire:** `hqsicccccc`: target runtime ID:q, target name:s (capacity 4000 UTF-16 units), correlation:i, option A:c, difficulty:c, options B/C/D/E:c. ❓ The option names remain open.
- **Checks/order:** Decode, else 159/2. Require PC user +20696, PC +20752 clear and user +120 dword zero, else 159/754. Normalize difficulty outside 1..3 to default byte at rule-config global `0x140F77978` +61880. Resolve target by nonzero runtime ID through `0x1403F7D20(...,6)`, otherwise by name through `0x1403F7EF0(...,6)`. Missing target uses 159/747.
- **STOP:** The next checks call excluded restriction helper `0x14060E280`. Do not follow its implementation. Its result branches select 747 or 755. Call-site addresses are listed in the boundary table below.
- **Ordinary continuation:** Reject equal runtime IDs or equal PC DB IDs (747). When requester has no party and requested difficulty differs from the default, reject source zone category +40=3 with subtype +48 in {26,34,38,77,177} (758). Under config +237 set and +248 zero, reject target arena state in {2,3,5,6} (6). Outside arena config, an existing cross-server registration triggers command 493, `hQqqs`: byte 2, requester DB ID, registration +120 qword, requester name. Route through `0x1400D0FC0` with PC world ID +96 and registration +136 dword. ❓ Destination daemon identity remains open.
- **Queue/state:** Require target zone (751). Create task vtable `0x140C3B198`. Require target kind 1. Attach to **target**, not inviter. Store inviter DB ID +144, inviter runtime ID +152, correlation +160, option A +164, options B/C/D/E +165..168, normalized difficulty +169. Dispatch through `0x140603BD0(target,task,0,0)`. Kind/dispatch failure uses reason 2.
- **Failure output:** Send 159, `hEqsicc`, to requester: reason, target runtime ID, target name, correlation, option A, difficulty. Early decode/user/missing-target failures echo decoded target fields. Resolved-target late failures use authoritative target ID/name. The visible post-resolution failure tail calls `0x14060B260`; ❓ this cleanup boundary was not read. Return true. No direct connection kick.
- **❓ Missing:** Restricted result policy, cleanup boundary, dispatch helper, invite task execution/timeout, party creation/authorization, invitation sends, success broadcasts and downstream daemon/DB work. No distance or cooldown rule is established by this wrapper.

## 161 — G_JoinPartyReqCmd — partial

- **381:** `0x14038D450`. **PDB hint:** `HandleGame_JoinPartyReq`.
- **Wire:** `hqqic`: first opaque qword, invitation/inviter qword, correlation:i, difficulty:c. The first body qword is decoded but not passed to the task. ❓ Confirm both qword meanings.
- **Checks/order:** Decode (162/2); require user present, PC +20752 clear and user +120 zero (768); difficulty in 1..3 unsigned (2); zone exists (764). Outside arena config, emit the optional command-493 registration request described under 157. This happens before task dispatch.
- **Queue/state:** Create task vtable `0x140BB0600`; require PC kind 1; attach; store second qword +144, correlation +152, difficulty +156. Dispatch through `0x140603BD0(PC,task,0,0)`. Kind/dispatch failure uses 2.
- **Output:** Failure sends 162, `hEic`: reason, correlation, difficulty, to self. Return true. No immediate success, broadcast or direct kick. Decode failure can echo uninitialized native correlation/difficulty bits; do not invent them.
- **❓ Missing:** Qword field meanings, task authorization/invitation match, member insertion, leader/rule inheritance, sync packets, late failures, daemon callback and DB work.

## 163 — G_RejectPartyReqCmd — partial

- **381:** `0x14038E350`. **PDB hint:** `HandleGame_RejectPartyReq`.
- **Wire:** `hqqcQ`: first qword, second qword, correlation/slot:c, rejection-kind:Q. All body locals start at zero. ❓ Semantic names remain open. The first qword is unused after decode.
- **Checks/order:** Decode (164/2); zone exists (164/764); allocate through `0x14036D780`; initialize through `0x1402B1D20(task,zone,PC,1,second qword,byte,kind)`; queue. Allocation/init/queue failure uses 164/2.
- **Output:** Failures send 164, `hEc`: reason and correlation/slot byte, to self. Return true. No immediate success, broadcast or direct kick.
- **State/dependencies:** Wrapper only creates task/child state. No direct table, daemon request or DB procedure.
- **Shared rejection task `0x1402B1E00`:** Resolve inviter runtime ID with rights 6. Missing inviter sends nothing and changes nothing. For group 1, search the caster's card box at PC +19712 with `0x1401F5610`: card type +24=1, inviter DB ID +0 and runtime ID +8 must both match the resolved PC. Remove only the first match, shift the later pointers, decrement count +19732 and free the card. A match sends inviter 186, `hqscQ`: caster runtime ID, caster name, correlation byte and original rejection-kind byte. No match sends nothing. No self success reply occurs.
- **Group 2:** Search types 2, 3 and 4 in that order. Remove the first match of each type. If any card was removed, send inviter 264, `hqsQ`: caster runtime ID/name and rejection-kind byte. Send only one notification. No kind range, card-expiry or timeout check occurs. The request's first qword is unused; its second qword is the inviter runtime ID. The correlation byte is an opaque echoed field.
- **Reports:** If inviter lookup succeeds, config +537 can send a rejection report through `0x140203710`. The report runs even without a matching card. Daemon +2728 and pointee +32 enable a second game-log callback (event 46). These calls do not supply a reason or undo removal. Rust stops at these report adapters after the verified removal and notification effects. No party/team membership changes or DB procedure occurs.
- **Completion:** All game-logic paths are read. Rejection kind passes through unchanged. Sources: build 381.8867, `0x1402B1D20`, `0x1402B1E00`, `0x1401F5610`.

## 243 — G_LeaveTeamReqCmd — partial

- **381:** `0x140391B40`. **PDB hint:** `HandleGame_LeaveTeamReq`.
- **Wire:** `hQ`: leave-mode byte, initialized to zero.
- **Checks/order:** Decode (244/2); team-system config +725 enabled (2); zone exists (764). If mode=1, create the same leave-party task as 168 and initialize with `(0,0)`. For **every other byte**, create task vtable `0x140B9A460`; require PC kind 1; attach; task +144 qword=0 and +152 dword=0. Queue either task. Allocation/init/kind/queue failure uses 2. Do not restrict mode to 0..1 without later evidence.
- **Output:** Wrapper failure is 244, `hEQ`: reason and original mode, to self. Return true. Mode 1 can later use the party task's replies, not command 244.
- **State/dependencies:** Task/child state only. No immediate success, broadcast, direct kick, table, daemon or DB call.
- **Task `0x1402B2A90` (mode !=1):** Suppress manager call only for arena config, daemon +4152 byte=2 and PC zone subtype 164. Otherwise call `0x1401D5F50(manager,PC,0,0)`. Nonzero returned short sends 244, `hEQ`, with that reason and **literal mode 2**, not the original request byte. Zero sends no direct task reply.
- **❓ Missing:** Manager removal/disband/leader rules, notifications and backend effects. Mode-1 continuation shares the gaps under 168.

## 245 — G_BanishTeamReqCmd — partial

- **381:** `0x140392120`. **PDB hint:** `HandleGame_BanishTeamReq`.
- **Wire:** `hhc`: party index:h, member slot:c. Both body values start at zero.
- **Checks/order:** Team-system config +725; decode; zone; create task vtable `0x140BBAA80`; PC kind 1; attach; task +144 short=party index, +146 byte=member slot; queue. Each wrapper failure sends 246, `hh`, reason 2, to self. Return true.
- **State/dependencies:** Task/child state only. No immediate success, broadcast, direct connection kick, table, daemon or DB call.
- **Task `0x1402B2C00`:** Require nonzero PC party ID +21112, else 246/781, `hh`. Call `0x1401D7FD0(manager,PC,unsigned party short,unsigned member byte)` on success.
- **❓ Missing:** Manager role/index/target checks, removal, notifications and backend effects.

## 247 — G_TeamMemberPartyChangeReqCmd — partial

- **381:** `0x1403923C0`. **PDB hint:** `HandleGame_TeamMemberPartyChangeReq`.
- **Wire:** `hhcqhcq`: source party:h, source slot:c, source runtime ID:q, destination party:h, destination slot:c, destination runtime ID:q. ❓ Source/destination names are inferred from the operation; exact exchange semantics remain open.
- **Checks/order:** Config +725 enabled (249/2); decode (2); reject source party raw 0xFFFF, source slot raw 0xFF or source ID zero (878); zone exists (2); create task vtable `0x140BC1FA8`; PC kind 1; attach; queue (2). There is no matching destination-field gate in this wrapper.
- **Task fields:** Source party +144 short, slot +146 byte, ID +152 qword; destination party +160 short, slot +162 byte, ID +168 qword.
- **Output:** Wrapper failure sends 249, `hE`, to self. Return true. No immediate success or broadcast.
- **State/dependencies:** Task/child state only. No direct kick, table, daemon or DB call.
- **Task `0x1402B2CE0`:** Require nonzero PC party ID +21112, else 249/880, `hE`. Call `0x1401D8C50(manager,PC,source party,source slot,source ID,destination party,destination slot,destination ID)`.
- **❓ Missing:** Manager leader authority, index/identity validation, move/swap semantics, leader/rule effects, sync and backend work.

## 250 — G_DelegateTeamLeaderReqCmd — partial

- **381:** `0x1403927C0`. **PDB hint:** `HandleGame_DelegateTeamLeaderReq`.
- **Wire:** `hq`: target runtime ID, initialized to zero.
- **Checks/order:** Config +725 enabled (252/2); decode (2); zone exists (764); create task vtable `0x140BBD3D8`; PC kind 1; attach; store target +144; queue. Kind/queue failure uses 2.
- **Output:** Wrapper failure sends 252, `hE`, to self. Return true. No immediate success or broadcast.
- **State/dependencies:** Task/child state only. No direct kick, table, daemon or DB call.
- **Task `0x1402B2E40`:** Require PC party ID +21112 nonzero, else 252/887, `hE`. Resolve target runtime ID through `0x1403F7D20(...,6)`, else 252/889. Call `0x1401E1580(...,PC,target)` and release target.
- **❓ Missing:** Manager current-leader/target checks, leader mutation, notifications and backend effects.

## 253 — G_NominatePartyLeaderReqCmd — partial

- **381:** `0x140392A30`. **PDB hint:** `HandleGame_NominatePartyLeaderReq`.
- **Wire:** `hq`: target runtime ID, initialized to zero.
- **Checks/order:** Config +725 enabled; decode; zone; create task vtable `0x140BAC710`; PC kind 1; attach; store target +144; queue.
- **Output:** Disabled config sends **252/2**, not 255. Decode/zone/kind/queue failure sends 255/2. Formats are `hE`, to self. Return true. No immediate success or broadcast.
- **State/dependencies:** Task/child state only. No direct kick, table, daemon or DB call.
- **Task `0x1402B2F90`:** Require PC party ID +21112 nonzero, else 255/891, `hE`. Resolve target through `0x1403F7D20(...,6)`, else 255/890. Call `0x1401E19F0(...,PC,target)` and release target.
- **❓ Missing:** Manager team/party role and target checks, party-leader update, notifications and backend work.

## 256 — G_ChangeTeamInviteAuthReqCmd — done

- **381:** `0x140392CA0`. **PDB hint:** `HandleGame_ChangeTeamInviteAuthReq`.
- **Wire:** `hQ`: authority byte, initialized to zero.
- **Checks/order:** Config +725 enabled (258/2); decode (2); byte <=2 unsigned (894); zone exists (764); create task vtable `0x140BC1720`; PC kind 1; attach; store byte +144; queue. Kind/queue failure uses 2.
- **Output:** Failures send 258, `hh`, to self. Return true. No immediate success or broadcast.
- **State/dependencies:** Task/child state only. No direct kick, table, daemon or DB call.
- **Task `0x1402B30E0`:** Require PC party ID +21112 nonzero. Missing membership sends **173/781**, `hh`, not 258. Otherwise call `0x1401E1DD0(...,PC,authority)`.
- **Manager `0x1401E1DD0`:** Resolve locked team/pair; failure sends 258/895, `hh`. Require both team leader DB ID +112=PC DB ID and runtime ID +104=PC runtime ID; mismatch also sends 258/895. Set team +120 dword to authority. There is no unchanged-value rejection. Compose 274, `hQ`, with authority; iterate team party indexes at +88, count +52; validate each index against team +48 and vector +56; send through `0x1402132A0` for each found party. Unlock/release team. No direct success ack, table, daemon, DB or kick in manager.
- **Completion:** All game-logic paths are read. Use the exact resolver/member and party broadcast contracts above. No DB procedure, other-daemon request, distance or cooldown gate occurs on this operation.

## 259 — G_ChangeTeamPartyDifficultyTypeReqCmd — partial

- **381:** `0x140392F50`. **PDB hint:** `HandleGame_ChangeTeamPartyDifficultyTypeReq`.
- **Wire:** `hcc`: party selector byte, difficulty byte. Both start at zero.
- **Checks/order:** Config +725 enabled (261/2); decode (2); difficulty in 1..3 unsigned (2); zone exists (764); create task vtable `0x140BC1680`; PC kind 1; attach; store selector +144 and difficulty +145; queue. Kind/queue failure uses 2. No wrapper selector range check.
- **Output:** Failures send 261, `hh`, to self. Return true. No immediate success or broadcast.
- **State/dependencies:** Task/child state only. No direct kick, table, daemon or DB call.
- **Task `0x1402B31A0`:** Require PC party ID +21112 nonzero, else 261/897, `hh`. Otherwise call `0x1401E1FC0(...,PC,unsigned selector,unsigned difficulty)`.
- **Manager `0x1401E1FC0`:** Resolve locked requester team/pair, else 261/897. Require both leader DB ID and runtime ID to match requester, else 897. Interpret selector as signed byte; a negative index, out-of-range index or null party gives 261/2. Call `0x1402140D0(selected party,0,difficulty,1)`. Use all capacity, unchanged and content gates under 179, with member index forced to zero. Nonzero reason sends 261 `hh` and stops; no team sync runs on failure. On success, metadata dispatch includes all selected-party members, then the optional difficulty report runs. Broadcast 275 `hcc` (selector, difficulty) to indexed team parties. Then send self 260 **`hc`**, with selector only. The native call passes an extra difficulty argument; the descriptor does not serialize it. No team-mode gate occurs on this success broadcast.
- **Completion:** All game-logic paths are read. Use the metadata task, report boundary and recipient contracts under 179. No other daemon or gameplay DB request occurs. Sources: build 381.8867, `0x1401E1FC0`, descriptor `0x1408D9508`, `0x1402140D0`, `0x14021EDE0`.

## 147 — G_SuggestPartyReqCmd — partial

- **381:** `0x14038BE90`. **PDB hint:** `HandleGame_SuggestPartyReq`.
- **Wire:** `hqscccccc`: target runtime ID:q, target name:s (capacity 4000 UTF-16 units), option A:c, difficulty:c, options B/C/D/E:c. Decode return is ignored. Only difficulty and name storage are explicitly zeroed.
- **Checks/order:** Reject request runtime ID equal to self (150/740). Resolve target by nonzero runtime ID or by name, as under 157; missing target or resolved self also uses 740. If target has no party, requested difficulty differs from default +61880, and target zone category=3/subtype in {26,34,38,77,177}, use 741. With config +237 set/+248 zero, target arena state in {2,3,5,6} uses 6. No difficulty normalization occurs here.
- **Direct-send branch:** If daemon rule config +512 byte is zero, send target 149, `hscccccc`: requester name and all six bytes. Then send self 148, `hqscc`: original target ID/name, option A and difficulty. No party mutation or pending-invite insertion is shown in this branch.
- **Task branch:** Otherwise require target zone; create task vtable `0x140BA8220`; require target kind 1; attach to target; task +144 requester runtime ID, +152..157 six bytes in wire order; queue. Zone/kind/queue failure uses 150/2.
- **Failure:** 150, `hEqscc`: reason, original target ID/name, option A, difficulty, to self. Return true. No direct kick, table lookup, daemon request or DB procedure in wrapper.
- **Task `0x1402B4350`:** Resolve requester runtime ID with rights 6. Missing requester logs only. If caster has no party, send caster 149 with requester name and six bytes. Then send requester 148 with the caster's authoritative ID/name and first two bytes. Otherwise resolve caster's locked team/member pair. Failed resolution sends nothing. Call `0x1401EBB50` on success. Unlock/release after the call.
- **Routing `0x1401EBB50`:** Config +2736 byte +732 selects team-aware routing. When clear, validate the party and call `0x1402133B0`. Invalid party sends requester 150/740 with caster ID/name and first two bytes. When set, team mode 1 uses `0x1401F31D0(team,2,party byte,caster DB ID,caster runtime ID)`. Other modes require a valid party; authority 0 permits member index 0, authority 1 permits every member, other authorities do not permit direct notification. A permitted caster receives 149; requester receives 148 with caster ID/name. Otherwise resolve team leader with rights 6. Missing leader or leader arena state {2,3,5,6} under +237 set/+248 clear sends 150/740 with caster ID/name. Success sends leader 149, then requester 148 with leader ID/name.
- **Party routing `0x1402133B0`:** Authority 0 selects member index 0. Authority 1 selects caster member index. Other values fail. Zero selected ID, selected ID=requester ID, missing selected PC, or selected PC arena gate as above gives 150/740 with caster ID/name. Otherwise send selected PC 149, then requester 148 with selected ID and authoritative selected name. No invite card, timer, mutation or backend call occurs.
- **Completion:** All game-logic paths are read. Config +512 is the task-routing switch; no symbolic name is needed for the byte contract. Option bytes remain opaque and pass through unchanged. ❓ Native uninitialized values on short unchecked decode have no stable contract; Rust stops at that boundary. Source: build 381.8867, the functions above.

## 151 — G_RejectSuggestPartyReqCmd — partial

- **381:** `0x14038C640`. **PDB hint:** `HandleGame_RejectSuggestPartyReq`.
- **Wire:** `hqscc`: target runtime ID:q, target name:s (capacity 4000 UTF-16 units), option A:c, difficulty:c. Decode return is ignored. Name storage is zeroed; other malformed-input values remain unspecified.
- **Checks/order:** Reject request ID equal to self. Resolve by nonzero runtime ID, otherwise name. Reject missing target or resolved self. All these failures send 154, `hEqscc`, reason 742, followed by original target ID/name and two bytes.
- **Success:** Send target 153, `hscc`: requester name and two bytes. Then send self 152, `hqscc`: original target ID/name and two bytes. Do not replace echoed ID/name with resolved values.
- **State/dependencies:** No party mutation, membership/leader/distance/cooldown/table gate, task, daemon, DB or direct kick in the read wrapper. Return true.
- **Completion:** All game-logic paths are read. The shared world/name contracts specify eligibility and CRT `wcsicmp`. Option bytes pass through unchanged. ❓ Short unchecked decode can expose uninitialized native fields; Rust returns unsupported and sends nothing there. Source: build 381.8867, `0x14038C640`, `0x1403F7EF0`, `0x140433BB0`.

## 226 — G_SuggestTeamReqCmd — partial

- **381:** `0x14038FC00`. **PDB hint:** `HandleGame_SuggestTeamReq`.
- **Wire:** `hQqsccccc`: suggestion mode:Q, target runtime ID:q, target name:s (capacity 4000 UTF-16 units), five option bytes:c. Decode return is ignored. Only name storage is zeroed.
- **Checks/order:** Decode; require config +725; reject request target ID=self; resolve target by ID/name; require target. Disabled/self/missing uses 229/822. Replace target ID with resolved runtime ID. Under config +237 set/+248 zero, target arena state {2,3,5,6} uses reason 6. Require target +21136 byte nonzero and +21104 qword nonzero; resolve locked team/pair, else 822.
- **Recipient selection:** Map suggestion mode 1/2/3 to permission mode 2/3/4; other bytes use permission mode 0. Call `0x1401F31D0(team,permission mode,low-byte party index,target DB ID,target runtime ID)`. If true, notify target. Otherwise resolve team leader runtime ID +104 and notify that PC; missing leader uses 822.
- **Output:** Recipient gets 228, `hQsccccc`: suggestion mode, requester name, five option bytes. Self gets 227, `hQqsc`: suggestion mode, recipient runtime ID, target name, first option. On leader reroute, self receives the leader's authoritative name instead of target name. Failures send 229, `hEQqsc`: reason, mode, target ID/name, first option. Return true.
- **State/dependencies:** No task or direct party mutation. Team lock spans permission/recipient sends. No direct kick, table, daemon or DB call.
- **Permission `0x1401F31D0`:** Both leader identities match → true before the authority gate. Authority 0 → false. Authority 1 → require signed party byte >=0, a valid party pointer, both identities equal to member index 0, and permission mode 2 or 3. Authority 2 → a matching party leader needs mode 2 or 3; every other requester needs mode 2. Other authority values → false. No member-state gate occurs inside this predicate. The caller already resolved target membership in mode 3.
- **Completion:** All game-logic paths are read. Use the shared case-insensitive resolver and exclusive state contract. Mode and option bytes pass through unchanged; their labels add no gate. ❓ Short unchecked decode can expose uninitialized native values; Rust stops there. Source: build 381.8867, `0x1401F31D0`, `0x14038FC00`, `0x140433BB0`.

## 230 — G_RejectSuggestTeamReqCmd — partial

- **381:** `0x140390370`. **PDB hint:** `HandleGame_RejectSuggestTeamReq`.
- **Wire:** `hQqscc`: mode:Q, target runtime ID:q, target name:s (capacity 4000 UTF-16 units), two option bytes:c. Decode return is ignored. Only name storage is zeroed.
- **Checks/order:** Decode; config +725; request target ID differs from self; resolve by ID/name; require target. There is **no resolved-self check** after name resolution in this function.
- **Output:** Disabled config sends **229**, `hEQqsc`, reason 822, mode, original target ID/name, first option. Self-ID or missing target sends 233, `hEQqscc`, reason 823, mode, target ID/name and both options. Success replaces target ID with resolved runtime ID, sends target 232, `hQscc` (mode, requester name, two options), then self 231, `hQqscc` (mode, resolved target ID, original target name, options). Return true.
- **State/dependencies:** No task, party mutation, role/distance/cooldown/table check, daemon, DB or direct kick in wrapper.
- **Completion:** All game-logic paths are read. The shared world/name contracts specify eligibility and CRT `wcsicmp`. Mode and option bytes pass through unchanged. ❓ Short unchecked decode can expose uninitialized fields; Rust returns unsupported there. Source: build 381.8867, `0x140390370`, `0x1403F7EF0`, `0x140433BB0`.

## 234 — G_InviteTeamReqCmd — partial

- **381:** `0x140390760`. **PDB hint:** `HandleGame_InviteTeamReq`.
- **Wire:** `hQqsccccc`: invite mode:Q, target runtime ID:q, target name:s (capacity 4000 UTF-16 units), five option bytes:c. All body values start at zero.
- **Checks/order:** Decode (236/2); config +725 (2); user exists, PC +20752 clear, user +120 zero (840); mode in 1..3 (825); resolve target by ID/name (824). Next comes the excluded restriction boundary `0x14060E280`; result mismatch paths use 824 or 841. Do not follow the helper. Replace target ID with resolved ID. Reject same runtime ID or same PC DB ID (824). Under config +237 set/+248 zero, reject target arena states {2,3,5,6} (6). Outside arena config, emit optional registration command 493 as under 157 before the mode checks below.

| Mode | Destination task caster | Failure reasons after target resolution |
| ---: | --- | --- |
| 1 | Target PC | No extra membership gate in wrapper. |
| 2 | Leader of target party, through `0x1401E0900` then world PC lookup | Requester party ID +21112 zero:829; target party ID zero:837; missing leader ID or PC:835. |
| 3 | Leader of target team, through `0x1401D4130` and team +104 then world PC lookup | Requester party ID zero, +21136 byte zero or +21104 qword zero:829; target +21136 zero or +21104 zero:836; missing team/leader/leader PC:834. |

- **Queue/state:** Require selected caster zone (832). Create task vtable `0x140BB0230`. Require caster kind 1. Attach to selected caster. Task +144 mode; +152 requester DB ID; +160 requester runtime ID; +168..172 five option bytes; +176 original resolved target runtime ID. Dispatch through `0x140603BD0(caster,task,0,0)`. Kind/dispatch failure uses 2.
- **Output:** Failure sends 236, `hEQqsc`: reason, mode, target ID/name, first option, to self. Pre-resolution failures echo original fields. Mode/zone/task late failures use resolved target ID/name. Return true. No immediate success or connection kick.
- **❓ Missing:** Exact option names, restricted policy, party/team leader-resolver internals, task authority/capacity checks, team conversion/merge, invitation/timeout/sync, command-493 daemon identity/callback and DB effects. No distance/cooldown rule is established by the wrapper.

## 239 — G_JoinTeamReqCmd — partial

- **381:** `0x140391630`. **PDB hint:** `HandleGame_JoinTeamReq`.
- **Wire:** `hQqq`: mode:Q, inviter/invitation runtime ID:q, second qword. All body values start at zero. ❓ Exact qword names remain open.
- **Checks/order:** Decode (240/2); config +725 (2); user exists, PC +20752 clear, user +120 zero (863); first qword differs from own runtime ID (2); zone exists (852). Outside arena config, emit optional registration command 493 as under 157.
- **Queue/state:** Create task vtable `0x140C406F0`. Require PC kind 1. Attach; store mode +144, first qword +152, second qword +160. Dispatch through `0x140603BD0(PC,task,0,0)`. Kind/dispatch failure uses 2. No mode range check appears in wrapper.
- **Output:** Failure sends 240, `hEQ`: reason and original mode, to self. Return true. No immediate success, broadcast or direct connection kick.
- **❓ Missing:** Deferred invitation/mode/role/capacity checks, team creation/conversion/merge, member changes, sync, registration callback and DB effects.

## 241 — G_RejectTeamReqCmd — partial

- **381:** `0x140391F50`. **PDB hint:** `HandleGame_RejectTeamReq`.
- **Wire:** `hqQ`: inviter/invitation qword and rejection-kind byte. Both start at zero. ❓ Exact meanings remain open.
- **Checks/order:** Decode; zone; allocate shared rejection task through `0x14036D780`; initialize through `0x1402B1D20(task,zone,PC,2,qword,0,kind)`; queue. Every wrapper failure sends 242, `hE`, reason 2, to self. Config +725 is not checked in this wrapper.
- **State/dependencies:** Task/child state only. Return true. No immediate success, broadcast, table, daemon request, DB procedure or direct kick.
- **Completion:** All game-logic paths are read. Use the group-2 card removal and command-264 contract under 163. The qword is the inviter runtime ID. The kind byte passes through unchanged. No team-config, expiry, invitation-token or membership gate occurs in the shared task. Optional report outputs remain explicit Rust adapters, not successful backend work. Sources: build 381.8867, `0x140391F50`, `0x1402B1D20`, `0x1402B1E00`.

## 216 — G_PartyVoteCastReqCmd — partial

- **381:** `0x14038F6C0`. **PDB hint:** `HandleGame_PartyVoteCastReq`.
- **Wire:** `hcQ`: vote target/member byte:c, vote byte:Q. Decode return is ignored. ❓ This name does not establish a general ready-check operation.
- **Checks/order:** If rule config +513 byte is zero, log only. Otherwise require arena config (+237 and +248 nonzero); failure sends **173/813**, `hh`, to self. Decode; require zone; create task vtable `0x140B98718`; require PC kind 1; attach; store bytes +144/+145; queue. Zone/kind/queue failure sends 219/2, `hh`, to self.
- **State/output:** Wrapper creates task/child state only. Return true. No immediate success, broadcast, table, daemon, DB or direct kick.
- **Task `0x1402B5540`:** Missing PC party ID sends self 219/812 `hh`. PC +21136 nonzero AND +21104 nonzero sends 219/814. Require PC qword +20816 plus signed config dword +516 <= global `Time`, else 219/818. Resolve locked team/member pair, else 219/812. Call `0x14021F1C0`; invalid party pointer/index gives 816. A nonzero result sends 219 `hh`. Zero sends 217 `hcQ` with the two task bytes. Unlock/release team. If the helper's output flag equals 1, call `0x1401D71D0(manager,PC,target slot,1)`. Then write current `Time` to PC +20816, even after a helper rejection. No timestamp write occurs on the earlier task gates. Source: build 381.8867, vtable `0x140B98718` +72.
- **Vote helper `0x14021F1C0`:** Set party +1812 byte=1 before any gate. Party +1760 signed dword >0 gives 815. Target slot equal to caster member slot gives 816. Save caster's old voted-for/vote bytes +56/+57, then replace both with the request bytes **before** the remaining checks. Count state-3 members other than the target; count their target votes equal to 1. If a state-3 target exists and rule config +514 is zero, clear the output flag and return 819. Fewer than two other state-3 members gives 817. No matching target slot gives 816. These failures do not undo the caster's vote. On success, resolve the state-3 target and send 218 `hcQ` with the new bytes. Resolve the old state-3 target and send 222 `hcQ` with the saved bytes. Missing PCs skip only those sends. Set output flag=1 when the affirmative count is at least the other-member count. Return zero. No request vote-byte range check occurs.
- **❓ Missing:** The exact global `Time` clock/width contract, symbolic config names and ban-manager removal effects remain open. Rust implements the first two task gates, then stops before the clock adapter. Native malformed-input bytes remain unspecified. Source: build 381.8867, `0x1402B5540`, `0x14021F1C0`, `0x1401D71D0`.

## 220 — G_PartyVoteCancelCastReqCmd — done

- **381:** `0x14038F9B0`. **PDB hint:** `HandleGame_PartyVoteCancelCastReq`.
- **Wire:** No body decoder.
- **Checks/order:** Rule config +513 byte zero logs only. Otherwise resolve locked team/member pair with `0x1401D3F70`; missing team sends 223/820, `hh`. Call `0x1401F2A30(team,&pair)`. Nonzero returned short sends 223, `hh`, with that reason. Zero sends 221, `h`, to self. Unlock/release team and log. Return true.
- **State/dependencies:** No task. Vote mutation is delegated to the helper. No direct kick, table, daemon or DB call in wrapper.
- **Cancel helper `0x1401F2A30`:** Validate party index/pointer, else 816. Require `party member count >= member index+1`, else 820. Requester's member record +56 byte is the stored voted-for slot; raw 0xFF means no vote (821). Scan member records for slot byte +2 equal to that stored byte. If found with state +24=3 and runtime ID +16 nonzero, resolve that PC. If present, send it 222, `hcQ`: old voted-for slot and old vote byte at requester record +57. Send occurs **before** clearing state. Then set requester's record +57=0 and +56=0xFF. Missing target, offline target or failed PC lookup does not prevent clearing or success.
- **Completion:** All game-logic paths are read. Use the exact resolver/member and world-ID lookup contracts above. No table, daemon, DB or kick occurs. This cancels a stored party-ban vote; it is not a general ready-state change.

## 214 — G_PassPartyAuctionReqCmd — partial

- **381:** `0x14038ECA0`. **PDB hint:** `HandleGame_PassPartyAuctionReq`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone; create task vtable `0x140B9C918`; require PC kind 1; attach; store raw PC pointer +144; queue. Each wrapper failure sends 215, `hh`, reason 2, to self. Return true.
- **State/dependencies:** Task/child state only. No immediate success, broadcast, party-role/distance/cooldown/table check, daemon, DB or direct kick in wrapper.
- **❓ Missing:** Task/auction identity and state checks, pass mutation, notifications, item/money handling and backend work.

## 205 — G_BidCurrentAuctionItemReqCmd — partial

- **381:** `0x14038F240`. **PDB hint:** `HandleGame_BidCurrentAuctionItemReq`.
- **Wire:** `hqc`: bid money:q, opaque auction byte:c. Decode return is ignored.
- **Checks/order:** Decode; require signed money >=0 and <= qword at global `0x140EE58A0` +32, else 207/33. Equality at both bounds passes. Require zone; create task vtable `0x140BC2608`; PC kind 1; attach; store PC pointer +144, money +152, byte +160; queue. Zone/kind/queue failure uses 207/2.
- **Output:** All failures use `hh`, to self. Return true. No immediate success or broadcast.
- **State/dependencies:** Task/child state only. The wrapper does not check current balance or minimum bid. No direct table, daemon, DB or kick.
- **❓ Missing:** Global maximum field name, byte meaning, task balance/minimum/auction-state checks, debit/refund, broadcasts, persistence and malformed native values.

## 202 — G_FinishPartyAuctionReqCmd — partial

- **381:** `0x14038F4C0`. **PDB hint:** `HandleGame_FinishPartyAuctionReq`.
- **Wire:** `hc`: opaque auction byte. Decode return is ignored.
- **Checks/order:** Decode; require zone; create task vtable `0x140BC2140`; PC kind 1; attach; store PC pointer +144 and byte +152; queue. Every wrapper failure sends 204, `hh`, reason 2, to self. Return true.
- **State/dependencies:** Task/child state only. No immediate success, broadcast, role/distance/cooldown/table check, daemon, DB or direct kick in wrapper.
- **❓ Missing:** Byte meaning, task finish/leader/item checks, award/money mutations, success/broadcasts, persistence and malformed native byte.

## 424 — G_RegisterPartyMatchReqCmd — partial

- **381:** `0x1403A1AC0`. **PDB hint:** `HandleGame_RegisterPartyMatchReq`.
- **Wire:** `hi`: match selector:i. Decode return is ignored. ❓ Exact selector meaning remains open.
- **Checks/order:** Decode; require user present, PC +20752 clear and user +120 zero, else 426/1320. Call `0x14017E540(manager at daemon +86864,selector,PC)`. Map result 0 to success; 1→1318, 2→1317, 4→1319, every other nonzero→2.
- **Output:** Success sends self 425, `h`. Failure sends self 426, `hh`, with mapped reason. Return true. No direct broadcast or kick.
- **Manager `0x14017E540`:** Require unsigned selector in 1..95, else result 2. Resolve selector through DataManager table slot 1983, virtual +168. Missing record gives result 2. Null PC gives result 1; authenticated-PC dispatch cannot reach this path. Record qword +24 is required faction: zero permits all factions; otherwise it must equal sign-extended PC byte +328, else result 4. Retain the PC. Under manager +48 lock, append a 16-byte entry containing selector dword +0 and PC pointer +8 to the deque at +88; increment queued count +120. Unlock and return 0. The selector is the table key, not a party ID. No role, party, cooldown, duplicate-registration, task-zone, daemon or DB gate occurs here.
- **Completion:** All registration-entry paths are read. Reply 425 means the request entered the matchmaker queue. It does not mean matching completed. The matchmaker consumer is outside this receive operation. Rust uses an unknown-default table adapter and a retained-PC queue adapter. ❓ Short unchecked decode can expose an uninitialized selector; Rust stops there. Sources: build 381.8867, `0x1403A1AC0`, `0x14017E540`, grow helper `0x14017EFA0`.

## 429 — G_RejectPartyMatchReqCmd — partial

- **381:** `0x1403A1C20`. **PDB hint:** `HandleGame_RejectPartyMatchReq`.
- **Wire:** `hqi`: target runtime ID:q, match correlation:i. Decode return is ignored. ❓ Correlation meaning remains open.
- **Checks/order:** Decode; resolve target via `0x1403F7D20(...,6)`. Missing target logs only. Found target receives 430, `hqi`: requester's authoritative runtime ID and original correlation. No self reply.
- **State/dependencies:** No match deregistration, party mutation, task, role/distance/cooldown/table check, daemon, DB or direct kick in wrapper. Return true.
- **Completion:** All game-logic paths are read. The i32 is an opaque correlation value; no instruction interprets it. Use the shared world-ID lookup contract. ❓ Short unchecked decode can expose uninitialized fields; Rust returns unsupported there. No membership or invitation-token gate occurs. Source: build 381.8867, `0x1403A1C20`.

## 174 — G_ClearLogoutPartyMemberReqCmd — partial

- **381:** `0x14038DF10`. **PDB hint:** `HandleGame_ClearLogoutPartyMemberReq`.
- **Wire:** No body decoder.
- **Checks/order:** Use the existing command-174 section in `10-session-handlers.md`. Require zone; create through `0x14036D800`; initialize through `0x1402B4920`; queue. Wrapper failures send 175/2, `hh`, to self. Return true.
- **State/output:** Shared task `0x1402B49D0` requires nonzero PC party ID, else 175/2. It calls `0x1401DD540` to remove expired members. No immediate success or direct kick. Existing manager evidence remains valid; do not duplicate its definition here.
- **❓ Missing:** Expiry predicate, mutation/leader/disband behavior, success/broadcast bodies and downstream daemon/DB effects. This job does not close the existing gaps.

## 262 — G_ClearLogoutTeamMemberReqCmd — partial

- **381:** `0x140393200`. **PDB hint:** `HandleGame_ClearLogoutTeamMemberReq`.
- **Wire:** No body decoder.
- **Checks/order:** Use the existing command-262 section in `10-session-handlers.md`. Require config +725; zone; create/init/queue the same task as 174. Wrapper failures send 263/2, `hh`, to self. Return true.
- **State/output:** Deferred task can still send **175/2** for missing party ID. The shared manager contains both 175 and 263 failure routes. No immediate success or direct kick.
- **❓ Missing:** Same shared manager, expiry/mutation/notification and downstream backend gaps as 174.


## 211 — G_DistributionCurrentAuctionItemReqCmd — partial

- **381:** `0x14038EE70`. **PDB hint:** `HandleGame_DistributionCurrentAuctionItemReq`.
- **Wire:** No body decoder.
- **Checks/order:** Require zone; create task vtable `0x140C36BB8`; PC kind 1; attach; store PC pointer +144; queue. Every wrapper failure sends 213, `hh`, reason 2, to self. Return true.
- **State/dependencies:** Task/child state only. No immediate success, broadcast, role/distance/cooldown/table check, daemon, DB or direct kick in wrapper.
- **❓ Missing:** Task leader/auction checks, distribution/item ownership, money allocation, replies/broadcasts and persistence.

## 208 — G_GiveUpCurrentAuctionItemReqCmd — partial

- **381:** `0x14038F040`. **PDB hint:** `HandleGame_GiveUpCurrentAuctionItemReq`.
- **Wire:** `hc`: opaque auction byte. Decode return is ignored. ❓ Exact field meaning remains open.
- **Checks/order:** Decode; require zone; create task vtable `0x140C43720`; PC kind 1; attach; store PC pointer +144 and byte +152; queue. Every wrapper failure sends 210, `hh`, reason 2, to self. Return true.
- **State/dependencies:** Task/child state only. No immediate success, broadcast, role/distance/cooldown/table check, daemon, DB or direct kick in wrapper.
- **❓ Missing:** Task byte/auction identity, give-up transition, notifications, item/money/backend effects and malformed native byte.


## 224 — G_PartyVoteResetReqCmd — done

- **381:** `0x14038FB50`. **PDB hint:** `HandleGame_PartyVoteResetReq`.
- **Wire:** No body decoder.
- **Checks/order:** PC dispatch. If rule config +513 is zero, log only. Otherwise call existing assertion/report helper `0x14069CB70` and write a log.
- **State/output:** Return true. No vote reset, task, reply, broadcast, party mutation, table lookup, daemon request, DB procedure or connection kick occurs in this function. Do not implement a reset from the request name.

## Rust port status

Source: the build-381.8867 paths and missing items in the sections above.
`partial` does not mean that a queued task completed its operation.
The parity tags use the section state in their anchors.
They use `ported` for the existing `done` sections, as in doc 19.

| IDs | Spec | Rust | Open boundary |
| --- | --- | --- | --- |
| 155, 182, 220, 224, 256 | done | ported | No unread game-logic path is listed. |
| 147 | partial | partial | Task and routing are implemented. Short unchecked decode and non-ASCII CRT locale need explicit boundaries. |
| 151 | partial | partial | Case-insensitive ASCII lookup is implemented. Short unchecked decode and non-ASCII CRT locale remain boundaries. |
| 157 | partial | partial | Excluded restriction call; cleanup and invitation task. No bypass adapter exists. |
| 161 | partial | partial | Invitation match, insertion, sync and backend work. This is not the shared rejection task. |
| 163, 241 | partial | partial | Exact first-card removal and 186/264 sends are implemented. Optional rejection report sinks remain unsupported. No timeout gate was found. |
| 168, 243 | partial | partial | Removal, leader replacement, notifications and backend work. |
| 170 | partial | partial | Party-leader mutation and team-leader transfer. |
| 172 | partial | partial | Task's 173/781 reply is implemented. Manager removal helpers, leader effects and notifications remain boundaries. |
| 174, 262 | partial | partial | Shared expired-member removal manager. |
| 176 | partial | partial | All game logic is read and implemented. Optional config +537 LogArchive report sink remains unsupported. |
| 179 | partial | partial | Capacity, blockers, metadata task dispatch/execution and team sync are implemented. Optional config +537 report sink remains unsupported. No resize occurs. |
| 202, 205, 208, 211, 214 | partial | partial | Auction task checks, transitions, item, money and persistence. |
| 216 | partial | partial | Early 219/812 and 219/814 replies are implemented. Rust stops before the clock, stored-vote/tally helper and ban continuation. |
| 226 | partial | partial | Permission predicate and recipient routing are implemented. Short unchecked decode and non-ASCII CRT locale remain boundaries. Missing state is unknown, not reason 822. |
| 230 | partial | partial | Literal mode/options and resolver behavior are implemented. Short unchecked decode and non-ASCII CRT locale remain boundaries. |
| 234 | partial | partial | Excluded restriction call; leader lookup and invitation task. |
| 239 | partial | partial | Invitation, mode, merge, member changes and backend work. |
| 245 | partial | partial | Team-ban manager checks, removal and notifications. |
| 247 | partial | partial | Member move/swap, leader rules and sync. |
| 250, 253 | partial | partial | Leader-manager checks, mutation and notifications. |
| 259 | partial | partial | Selected-party gates, all-member metadata dispatch, 275 broadcast and selector-only 260 ack are implemented. Optional config +537 report sink remains unsupported. |
| 424 | partial | partial | Selector/table/faction/queue logic is implemented. Table and retained-PC queue adapters default to unknown. Short unchecked decode remains unsupported. Reply 425 means queued only. |
| 429 | partial | partial | Authoritative requester ID and opaque i32 echo are implemented. Short unchecked decode remains unsupported. |

The Rust files are `src/handlers/party.rs`, `party_model.rs` and `party_tasks.rs`.
The tests are in `tests/pp1_party.rs`.
The party suite has 60 tests. This continuation adds 17 tests.
The tests cover the seven corrected assumptions, first-match rejection cards,
optional report boundaries, metadata dispatch and guarded PC writes, command-259
reply width/order, suggestion routing, CRT locale boundaries, match table/faction
checks and the new command-172/216 early task replies.
They do not verify the unread native paths.

### party2 verification notes

- The previous command-424 test expected selector 123 to reach an opaque manager. `0x14017E540` rejects that selector before the table read. The valid fixture now uses selector 23. Null-PC result 1 is unreachable through authenticated-PC dispatch. Table/faction and retained-PC queue tests replace arbitrary result injection.
- The previous command-226 tests injected permission results. `0x1401F31D0` now runs directly on team state. Tests cover each mode/authority and both party/team leader identities. The handler retains the locked state through recipient selection. It does not perform a third adapter read.
- The previous command-179 test expected an unread resize boundary. `0x14021EDE0` only dispatches kind-207 metadata tasks. Success tests leave config +537 off. Separate tests require unsupported status when a report sink is enabled.
- The previous name test required case-sensitive equality. `0x140433BB0` uses CRT `wcsicmp`. ASCII folding is implemented. Non-ASCII comparisons require the locale adapter. An absent adapter does not mean the target is missing.
- Missing metadata recipients must log and continue. Rust now retains failed lookup entries until the dispatch loop, as `0x14021EDE0` does.
- No test treats an invitation/join queue as completed membership work. No excluded restriction query was read or implemented.









