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
The existing BloodLib codec treats `K` as a u16 byte count followed by bytes.
Source: `Program/Server/BloodLib/src/packet.rs:171` and `:179`.
❓ The build-381 native `K` codec was not independently checked. Do not treat the existing codec alone as a compatibility test.

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
| 8 | 42 | Client → GameDaemon | `BUiWorld::RequestEnterZone`, `0x7815C0` | `hiuuuh` | Correlation value:i, three 16-bit destination fields, final short. Server `0x140371EE0`. |
| 9 | 43 | GameDaemon → client | Receive slot → `0xEF6790` → `World::EnterZone`, `0x74C850` | ❓ Not recovered | Client zone-entry completion target is verified. Server sender remains open. |
| 10 | ❓ | GameDaemon → client | ❓ Spawn/appear handlers not recovered | ❓ Not recovered | ❓ The local-player and nearby-object appearance set remains open. |

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
| six shorts | Configuration/session/world-related arguments | ❓ Full field labels and source mapping open |

The sender serializes property IDs 1 through `0x1A6` from the PC property container at +48.
It uses property writer `0x140697E50`.
❓ The property types, optional groups and total wire layout remain open.
A PC memory source does not by itself prove a character DB column.

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

The request constructor writes one dword and four shorts.
The server decodes `hiuuuh` in `0x140371EE0` and queues a task using the PC's zone.
It stores the first decoded dword at task +144.
❓ The dword is a candidate echo of the prepare correlation key. Exact client-to-server field correspondence remains open.
The client also calls `PresentationWorld::GetSummonedEnterZoneInfo` during request construction.
❓ Exact coordinate/counter meanings for all four shorts remain open.
Do not substitute a character ID for the first dword without further proof.

Command 42 is the checked request in the load/enter path.
❓ It is not proof of a separate universal loading-finished acknowledgement.
❓ The final load-screen exit condition, command-43 layout and spawn/appear exchange remain open.

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

No game record, account name, address or secret value is included.
See [06-minimum-session.md](06-minimum-session.md) for the conservative standing-session limit.
