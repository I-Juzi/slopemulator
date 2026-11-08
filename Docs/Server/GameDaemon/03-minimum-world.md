# Minimum world: 2020 GameDaemon data

## Byte evidence and standing chain

This document uses decoded file bytes only. Build: local BnS_2020 GameDaemon data, surveyed 2026-11-08. Paths are relative to that data folder. The table survey is `02-tables.md`.

| Source | Field or relation | Target | Evidence |
| --- | --- | --- | --- |
| `pcinitialsettingdata.xml`, PcInitialSetting record | race + job + level | Initial-setting selection key | 93 records; no id or alias |
| Same record | zone alias | zone | Level-one sample resolves to zone 1101 in `zone/1101/zonedata.1101.xml` |
| Same record | quest-1 alias | quest catalog | `quest/questdata.*.xml`; ❓ quest execution is not part of standing |
| `zone/1101/zonedata.1101.xml`, zone record | terrain id | Terrain | Sample resolves to terrain 1001 in `terraindata.1001.xml` |
| `zone/1101/zonepcspawndata.1101.xml`, ZonePcSpawn record | zone id; local id; pos; yaw | Initial player position | One spawn record in this sample; pos is three comma-separated numbers |
| zone record | default-phase-pc-spawn-id | ZonePcSpawn in that zone | ❓ Zero occurs. Zero is not an explicit spawn selection |
| zone record | reentrance-pcspawn | ZonePcSpawn alias across zones | `zone/1155/zonedata.1155.xml` names an alias in zone 2310; ❓ Reentry is not the verified initial-entry rule |
| zone record | respawn alias or zone.local-id | ZoneRespawn | `zone/1201/zonedata.1201.xml` uses composite numeric text. Resolve aliases across zones only when unique |
| ZoneRespawn record | area | ZoneArea key (zone.local-id) | ❓ Decimal-looking text is a composite reference, not a position or float |
| zone record; environment record | script alias | script catalog | Exact alias check only; no script execution |
| ZoneEnv record | zone id; action-point | zone; position | Legacy format: `zone/3045/zoneenvdata.3045.xml` |
| ZoneEnv2Spawn record | zone id; env2 alias; env2place alias | zone; ZoneEnv2; ZoneEnv2Place | `zoneenv2spawndata.xml`; ids restart in each zone |
| ZoneEnv2Place record | file zone scope; action-point | Position in zone | `zone/1101/zoneenv2placedata.1101.xml`; no id field |
| ZoneEnv / ZoneEnv2 | transit-zone; transit-pc-spawn | zone; zone-local spawn | Optional portal relation; ❓ zero/default semantics |
| zone record | phase-zone-N alias | zone | Optional phase relation |

The level-one sample starts in zone 1101 and uses terrain 1001. This is a byte-derived candidate, not a runtime observation. ❓ The server's selection rule for multiple initial settings or multiple spawns is not known. The loader must not choose the first record as a default.

## Table scope

| Table | Records in full survey | Typed fields |
| --- | ---: | --- |
| PcInitialSetting | 93 | race, job, level, inventory-size, zone, quest-1 |
| zone | 1533 | terrain, sector bounds, tutorial, default spawn, reentry, respawn, script, phase zones |
| Terrain | 164 | start, origin, counts, umap-name |
| ZonePcSpawn | 2951 | zone, local id, pos, yaw, summon-pos |
| ZoneRespawn | 1652 | zone, local id, area reference |
| ZoneArea | 76936 | zone, local id; remaining attributes retained |
| ZoneEnv | 98 | zone, local id, type, action-point, script, transit links |
| ZoneEnv2 | 11626 | alias, type, script, transit links |
| ZoneEnv2Place | 11682 | alias, file zone scope, type, action-point |
| ZoneEnv2Spawn | 11832 | zone, local id, env2, env2place |
| script | 17266 | alias catalog; all attributes and children retained |
| quest | 2610 | alias catalog; all attributes and children retained |

These tables cover the selected spatial chain. They do not resolve all spatial links or close every gameplay dependency. ❓ Environment items, rewards, effects, cinematics, region metadata, faction rules, NPCs, and quest/script behavior need further jobs. Unknown fields stay available; the loader does not claim to validate those links. Terrain records describe a map, not collision geometry. ❓ Coordinate units, yaw units, sector dimensions, and terrain resource files remain unverified. No path from a data attribute is opened.

## Loader contract (specification before implementation)

- Use the existing `lmxb` decoder. Do not parse plain XML in this crate.
- Walk directory entries in sorted order. Do not follow symlinks, including the root.
- Select tables by root `table` type, not by filename. Count skipped plain XML, unsupported tables, and other roots.
- Keep root metadata, record source paths and lines, unknown attributes, and all child nodes. Quest children use `quest`. Script children use `script` or `Script`. Three other direct script-table elements and table text stay in `TableFile.extra_nodes`. Evidence: `02-tables.md` element paths; `script/scriptdata.baekchung_kangryusi.xml` contains non-whitespace table text.
- Parse ids as unsigned integers. Parse signed terrain coordinates as integers. Parse positions as exactly three finite floating-point numbers. Preserve optional absence.
- Parse y/n flags as booleans. Reject other spellings for typed flags.
- Reject duplicate attributes, duplicate identity keys, malformed typed values, and missing required fields with source context.
- Index global ids and aliases for zone/Terrain/catalogs. Index zone-local ids and aliases for spatial records. Initial settings use race/job/level, not fabricated ids.
- Return all dangling references from the selected chain. Keep source table, source path, record line, attribute, target table, and key in each diagnostic.
- Treat absent links as absent. Treat numeric zero links as unspecified for checks only. ❓ This sentinel rule is inferred and does not prove runtime behavior.
- Resolve forward references only after loading all files. Do not fail the load for dangling references.
- An opt-in test reports counts and diagnostics. It must not assert zero dangling references in this incomplete data snapshot.

## Reproduction

```sh
cargo run --release --manifest-path bns-server/Cargo.toml -p lmxb --example tables --config 'env.BNS_GAMEDATA="/home/ubuntu/Downloads/BnS_2020/GameDaemon/data"' -- --out bns-server/Docs/Server/GameDaemon/02-tables.md
cargo test --release --manifest-path bns-server/Cargo.toml -p gamedata --test real_data --config 'env.BNS_GAMEDATA="/home/ubuntu/Downloads/BnS_2020/GameDaemon/data"' -- --nocapture
```

Without `BNS_GAMEDATA`, both entry points print `SKIP`. Tests must create their fixtures with `lmxb::encode`. Do not store game records in the repository.

## Verified loader results

The final opt-in test loaded 12 table types, 10,608 files, and 138,440 indexed records. The script catalog has 17,263 records. Its three other direct elements stay in file metadata. The full survey counts those three elements in its 17,266 direct-element total.

All 30 level-one initial settings resolve to zone 1101 and terrain 1001. Terrain links, initial zone/quest links, spawn-zone links, and respawn-area links have no unresolved references under the specified checks. This does not verify runtime entry behavior.

| Unresolved candidate link | Count |
| --- | ---: |
| zone phase-zone fields | 16 |
| zone reentrance-pcspawn | 24 |
| zone respawn | 2 |
| zone script | 593 |
| ZoneEnv script | 54 |
| ZoneEnv2 script | 5570 |
| ZoneEnv2 transit-pc-spawn | 9 |
| ZoneEnv2 transit-zone | 30 |
| ZoneEnv2Spawn env2 | 175 |
| ZoneEnv2Spawn env2place | 1974 |
| ZoneEnv2Spawn zone | 53 |
| Total | 8500 |

❓ These are unresolved candidates, not proof of corrupt data. Seven of the 1,974 unresolved placement aliases exist in another zone scope. The checker does not assume that a placement can be reused across zones. Missing script links may depend on other resources or naming rules. The bytes alone do not establish those rules.

The final workspace test run passed 290 tests, failed zero tests, and kept six ignored tests. This includes 14 new generated tests and the opt-in test's SKIP path. The separate real-data test passed. Workspace Clippy passed with `--workspace --all-targets -- -D warnings`. Formatting was applied to the two data crates.

