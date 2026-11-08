# 05 — Enter-world packets, build 381

## State

Partial static specification. No client test ran.
Use only the allowed build-381 client and GameDaemon databases for the addresses below.
The existing LobbyDaemon specification describes build 307. It supplies link context, not proof of identical build-381 code.
Connection and authentication are in [04-world-connect.md](04-world-connect.md).
Table dependencies are in [03-minimum-world.md](03-minimum-world.md).

## Packet identification

The client receive table starts at `0x149EC40` and uses command × 4.
Dispatcher `0xF44030` checks commands below `0x63F`.
GameDaemon dispatcher `0x14007BC00` reads the command at decoded packet +2.
Its unauthenticated table is `0x1407D6D50`.
Its authenticated-PC table is `0x1407D9F80`.
Server table entries use command × 8.

Client symbols are inconsistent. The named `HandleEnterZoneOkPacket` at `0xF3A860` contains unrelated UI code.
Use checked receive slots and command stores instead.
Names with ❓ below are descriptive names, not verified client symbols.
Packet-object offsets are not wire offsets.

Full format strings include the command `h`. They exclude the outer size field.
`h/u/E` use 2 bytes; `i` uses 4; `q` uses 8; `c/Q` use 1.
Scalars use little-endian order. Signedness depends on the field.
The existing BloodLib codec treats `K` as a nonnegative signed-16 byte count followed by bytes.
Source: `Program/Server/BloodLib/src/packet.rs:171` and `:179`.
Step 4 checked native `K` and `b` together in decoder `0x1406AA7B0`.
Both read a nonnegative signed-16 byte count, then that many bytes.
The decoder rejects a negative count, a short destination, or short input.
This matches the existing BloodLib byte-string codec.

## Ordered path

| Order | ID | Direction | Client symbol or checked target | Full format | Source and result |
| --- | ---: | --- | --- | --- | --- |
| 1 | 11 | Client → GameDaemon | Game-open callback `0xF2C0C0` | `hhqsqb` | Sends game Hello. See 04. |
| 2 | 12 | GameDaemon → client | `HandleHelloOkPacket`, `0xF70280` | `hb` | Processes opaque Hello reply. See 04. |
| 2a | 929 | Client → GameDaemon | Constructor `0xF2BEF0`, called by the command-12 callback | ❓ No application fields are visible in the checked constructor | ❓ Purpose, server response and relation to command 23 remain open. |
| 3 | 23 | Client → GameDaemon | ❓ Exact client sender not recovered | `hhqq` | world_id:h, client_key:q, lobby_token:q; server `0x140371170`. |
| 4 | 24 | GameDaemon → client | Receive slot → `0xF3AF90` | ❓ Complete body not recovered | Tests packet-object byte +0x10; constructs command 31. |
| 5 | 31 | Client → GameDaemon | Constructor in callback `0xF3AF90` | `hic` | Entry parameter:i, entry flag:c. Server `0x1403718A0`. |
| 6 | 32 | GameDaemon → client | Receive slot → `0xEF6860` → `World::EnterWorld`, `0x741A50` | Fragmented; checked prefix `hihqQihhhhhh…` | Initial PC composite. Sender `0x1405F5980`, fragment mode 2. |
| 7 | 41 | GameDaemon → client | Receive slot → `0xEF67D0` → world-interface prepare target `0x73E6C0` | `hqhiiccKuuuhhh` | Zone identity, destination and load data. Sender `0x1404BEED0`. |
| 8 | 42 | Client → GameDaemon | `BUiWorld::RequestEnterZone`, `0x7815C0` | `hiuuuh` | Spawn correlation:i, summoned-object x/y/z:uuu, summoned yaw:h. Server task `0x1402912F0`. |
| 9 | 43 | GameDaemon → client | Receive slot → `0xEF6790` → `World::EnterZone`, `0x74C850` | `hq` | Opaque runtime spawn-action value:q. Server sender `0x1402304A0`. |
| Unresolved order | ❓ | GameDaemon → client | Appearance queue call `0x14024B0C0` | ❓ Not recovered | Queue call precedes command 43. ❓ Packet IDs, bodies and delivery order remain open. |

This is a partial order, not a complete packet capture.
Command 12 does not prove a direct transition to command 23.
Supplementary packets can occur within and after the command-32 sender.
Do not infer their IDs from adjacent command numbers.

## Command 31: enter-world request

| Field | Width | Checked source | Meaning |
| --- | --- | --- | --- |
| Entry parameter | i | Client packet-object +0x10; server decode in `0x1403718A0` | Client starts with zero. A conditional security-version getter can change it. ❓ Server log calls it characterId. The conflict is unresolved. It is not a verified character DB ID. |
| Entry flag | c | Client packet-object +0x14, set to zero in the checked branch | ❓ Meaning not verified. |

The server rejects duplicate requests.
It calls the command-32 sender first.
It then selects a destination with `0x1405F1230` or an alternate route.
It calls world-entry helper `0x1403F4A60` after destination selection.
Thus a later destination failure can follow command 32.
The failure reply is command 33, full format `hh`, with reason:h.
Client command-33 receive slot points to `0xF2BEA0`.

The security getter is a boundary. Calls at `0xF3B00C` and `0xF3B1F6` are out of scope.
Do not inspect or reproduce that getter.

## Command 32: initial player composite

Sender: `0x1405F5980`. Format literal: `0x140DAA240`.
The literal starts `hihqQihhhhhh…`. This is a prefix, not an encodable complete format.
The sender uses fragment mode 2.
❓ The fragment-control field layout and limits were not recovered here.
Use the existing Packet fragment layer only after a compatibility check.
Do not send this payload as a header-only success.

| Checked prefix field | Native source | Provenance |
| --- | --- | --- |
| command:h | 32 | Protocol |
| fragment field:i | Fragment writer | Protocol; ❓ bit layout open |
| short:h | PC +96 | Runtime PC; ❓ semantic name and DB origin open |
| object_id:q | PC +40 | Runtime object identity; not a table ID |
| byte:Q | PC +13740 | Runtime PC; ❓ semantic name open |
| integer:i | Configuration +296 | Server configuration; ❓ semantic name open |
| short:h | Configuration +304 | Server configuration; ❓ meaning open |
| short:h | Runtime/config-selected local value | ❓ Meaning and original source open |
| short:h | Configuration +152 if +160 is enabled, else zero | Server configuration |
| short:h | World/daemon +86858 | Runtime world configuration; ❓ meaning open |
| short:h | Session +238, or zero | Runtime session; ❓ meaning open |

The descriptor contains six `h` fields after the configuration integer.
The initial compose call emits only the five independent shorts above.
❓ The next `h` appears to start the property section. Its property mapping is not verified.
Do not invent a sixth configuration field.

### Remaining ordered groups

Source: build-381 GameDaemon sender `0x1405F5980`, literal `0x140DAA240`.
This is a group inventory, not a complete encodable format.

| Order after prefix | Format / checked source | Provenance and limit |
| ---: | --- | --- |
| 1 | Property IDs 1..`0x1A6`; PC property container +48; writer `0x140697E50` | 422 property IDs. Descriptor advance is 373, not 422 wire fields. Custom `?` entries occur. ❓ Full dynamic layout and DB columns remain open. |
| 2 | `b`; helper `0x14025FB20` | Runtime PC byte block. ❓ Content identity open. |
| 3 | `K`; daemon pointer +3096, length +3104 | Daemon byte block. ❓ Original source and meaning open. |
| 4 | `C[ci]`; helper `0x140618730`, PC +3912 | Runtime collection. ❓ Element meanings open. |
| 5 | `C[Qb]`; count 2, tags 1 then 2 | Two property containers at PC +29808, stride 3352; property ID 2. ❓ Nested byte contract open. |
| 6 | `cccccq` | Four configuration bytes at +704/+708/+712/+716; helper byte `0x1405ED470`; PC q at +20744. ❓ Meanings open. |
| 7 | `C[Qqq]`; up to nine entries | Session +872/+880, stride 16. Emit nonzero pairs when configuration enables them. ❓ Semantic names open. |
| 8 | `Q` | Configuration +746 != 0. ❓ Option meaning open. |
| 9 | `C[ch]`; up to two entries | PC +39784/+39786, stride 4. Include only types 1..14. ❓ Meaning open. |
| 10 | ❓ `C[Qiqqihcccc]` from descriptor | Helper `0x1403E8C00`, session +1288. Helper-to-field mapping was not checked. |
| 11 | Final counted group | Helper `0x1401BB5E0`, PC +20616. ❓ Full element mapping remains open. |

A PC memory source does not prove a character DB column.
Keep the full command-32 body raw and unsupported.
The generic BloodLib codec cannot encode the custom `?` fields.
The existing fragment tests do not prove native mode-2 metadata compatibility.

The sender also composes nested data with `hchcccchchchch`.
A conditional composer uses `hqciqh`.
❓ These strings do not establish independent packet IDs or complete optional-group contracts.
Other send helpers run from this function.
❓ Inventory, skills, quests and social data were not separated from required initialization.

Client callback `0xEF6860` passes packet-object +0x500 to session virtual slot +0x38.
It then calls world-interface slot +4, which resolves to `World::EnterWorld`, `0x741A50`.
Stop the world-function path at the restricted reference `0x741AB2`.
The callback also reaches a security interface at `0xEF68C8`; do not follow that path.

## Command 41: prepare destination

The actual format bytes at `0x140CF1EF8` are `hqhiiccKuuuhhh`.
The IDA symbol spelling uses lowercase letters. The format byte is uppercase `K`.
Sender `0x1404BEED0` runs from PcSpawnAction start `0x1404BEAE0`.

| Body order | Type | Checked source | Field meaning / origin |
| ---: | --- | --- | --- |
| 1 | q | zone +40 | Zone runtime object ID; server runtime, not table key |
| 2 | h | zone +16600 | ❓ Zone-instance-related short; exact meaning open |
| 3 | i | First zone data record +8 | Zone table ID; table dependency below |
| 4 | i | Spawn action +208 | Nonzero spawn correlation key; server runtime |
| 5 | c | World configuration object +56 | ❓ Configuration byte; exact meaning open |
| 6 | c | Spawn action +193 != 0 | Action flag; ❓ meaning open |
| 7 | K | Pointer at zone +16200; length at +16208 | Zone byte block. These are two native arguments for one wire field. ❓ Block contents and source open. |
| 8–10 | uuu | Action +152/+154/+156 | Destination x/y/z, read as signed shorts by sender; ❓ units and table conversion open |
| 11 | h | Action +160 | Destination yaw; ❓ units open |
| 12 | h | Incremented PC +3480 | PC transition counter |
| 13 | h | Incremented summoned-object +3480, or zero | Summoned transition counter |

Entry helper `0x1403F4A60` resolves the destination zone and schedules PcSpawnAction through `0x14022EC20`.
The action stores position/yaw and generates the correlation key.
Its start checks or adjusts the position with `0x1404C1A70` before sending command 41.
❓ The saved-character location versus new-character table-selection branch is not fully mapped.

Client prepare target `0x73E6C0` requires a player object.
It caches destination/correlation data, calls `BPlayer::InitPos`, queries local map data, and notifies observers and `BParty::PrepareEnterZone`.
This establishes a player-initialization dependency before zone preparation.

## Loading and command 42

`UiTransitControl::enterWorld`, `0x7D8E70`, checks `World::IsDestZoneResource` and calls `UiTransitControl::loadZoneMap`, `0x78FED0`.
It creates the world UI and calls `0x78E8F0`.
That function reaches `BUiWorld::RequestEnterZone` at `0x78EC10` or `0x78EC39`.
A cinematic branch can defer this request.
Other checked callers are `0x7DA146` and `0x7DB4C3`.

The server decodes `hiuuuh` in `0x140371EE0` and queues task `0x1402912F0`.

| Body order | Type | Checked correspondence | Source class |
| ---: | --- | --- | --- |
| 1 | i | Task +144 must equal current spawn action +208, in action state 2. A mismatch sends command 44, error 1150. | Echo of command-41 runtime correlation. Not a character DB ID. |
| 2–4 | uuu | Task +148/+150/+152 pass to `0x1404BF580` for summoned-object x/y/z. | Client summoned-object state; not the local-player destination echo. |
| 5 | h | Task +154 passes to the same summoned-object branch as yaw. | Client summoned-object state. ❓ Units and valid ranges open. |

Client `0x7815C0` initializes the four shorts to zero and calls
`PresentationWorld::GetSummonedEnterZoneInfo`, `0xB523E0`.
The checked ordinary helper branch copies summoned-object shorts +0x94/+0x96/+0x98/+0x9A.
❓ No-summon and special-mode branches are not fully mapped.
The server can correct summoned position against terrain and permitted distance.
Keep raw scalar bits in the codec. Do not enforce an invented coordinate range.

Command 42 is the checked request in the load/enter path.
❓ It is not proof of a separate universal loading-finished acknowledgement.

## Command 43 and appearance order

Full format: `hq`. Server sender: `0x1402304A0`.
The only application field is a q value from spawn-action +184.
The source chain is `0x1402912F0` → `0x1404BF580` → `0x1404BF1E0` → `0x1402304A0`.
Use an opaque name, `spawn_context`. ❓ Its exact meaning remains open.
This is runtime action state, not a verified table key or character DB column.

The sender calls appearance-related queue helper `0x14024B0C0` before it sends command 43.
It calls event/script helper `0x140254770` after command 43.
Do not assume the wire order is always command 43 followed by all appearance packets.
❓ First appearance IDs, body layouts, DB/table inputs and delivery order remain open.

Client slot 43 selects `0xEF6790`, then `World::EnterZone` at `0x74C850`.
That target requires an existing player and starts installing it in the world object map.
❓ Final load-screen exit and a standing-client state remain unverified.

## Table and DB sources

| Entry data | Source class | Link to minimum-world specification | Limit |
| --- | --- | --- | --- |
| Admission token and client key | Lobby/session | Not game tables | Command 23 verified; command-24 body open |
| PC identity, appearance and properties | In-memory character/session state | ❓ Exact character DB columns and derived-property mapping open | No DB values copied |
| New-character destination candidate | PcInitialSetting → zone | [Standing chain](03-minimum-world.md#byte-evidence-and-standing-chain) | ❓ Binary selection branch not complete |
| Client terrain resource | zone → Terrain | Same standing chain | Terrain ID is not a separately verified field of command 41 |
| Spawn position and yaw candidate | ZonePcSpawn; or saved-character state | Same standing chain | ❓ Initial-spawn choice and conversion to wire shorts open; never choose the first record by default |
| Respawn/reentry | ZoneRespawn → ZoneArea; zone reentry relation | Same standing chain | ❓ Not proven required for first standing entry |
| Environment and nearby objects | ZoneEnv/ZoneEnv2/ZoneEnv2Place/ZoneEnv2Spawn | [Table scope](03-minimum-world.md#table-scope) | ❓ Appearance commands and mandatory subset open |
| Initial quests and scripts | PcInitialSetting quest link; script/quest catalogs | Same table scope | ❓ Not proven required or optional for standing |

## Rust codec contract (step 4)

| Item | Contract |
| --- | --- |
| Table | `Program/Server/Packet/tables/game381.json`; protocol 1278; separate client/server maps. This is a verified subset, not the complete native table. |
| Codec | `Program/Server/Packet/src/game.rs`; commands 11/12, 23, 31, 41/42/43, 16/17/18. Formats match the tables above. |
| Unknown payloads | Commands 24, 32, 929 and unknown IDs return `Error::Unsupported(RawPacket)`. The error retains command and all application bytes. Raw encoding also returns unsupported. |
| Packet boundary | One logical size/command/body packet; maximum 4000 bytes. No outer-frame crypto, native mode-2 assembly, dispatch authentication or world state machine is added. |
| Scalars | Unverified signed fields use unsigned raw-bit storage. Text retains UTF-16 units. No coordinate or game-value defaults are added. |
| Validation | Reject wrong directions, truncation, invalid counts and trailing fields. Generated tests check independent little-endian fixtures and round trips. |

The table has no LobbyGate relay mapping.
The codec does not verify admission tokens or process handshake secrets.
Do not infer safe state transitions from a successful codec operation.

No game record, account name, address or secret value is included.
See [06-minimum-session.md](06-minimum-session.md) for the conservative standing-session limit.
