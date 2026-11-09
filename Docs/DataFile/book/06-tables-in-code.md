# Chapter 6 — Tables in the client code

Build: `Client.exe` 0.0.381 x86, IDA copy `ida/lanes/cx/Client.exe.i64`.
✅ = verified in this copy. ❓ = not verified. Load steps are in chapter 5. Key packing is in 4.6.3.

## Quick find

| Question | Section | Address |
| --- | --- | --- |
| How does code get a table? | 6.1 | `dword_1A4D5AC + 52 + 52 × type` |
| Find a record by key (u64) | 6.2 | table `vtbl + 0x64`; `0x622d50`, `0x62d3d0` |
| Find a record by alias (name) | 6.3 | table `vtbl + 0x60`; `0x6307f0` |
| Go through all records | 6.4 | `0x622e00`, `0x62d4f0`, `0x62f570` |
| Which functions read which table? Top 60 | 6.5 | `tables.tsv` columns `readers`, `reader_count` |
| How does an id from a server packet become a client record? | 6.6 | `0x7422b8`, `0xd0a8be`, `0xd95842` |
| What does a Rust port need? | 6.7 | — |

## 6.1 Get a table

All tables hang on the global DataManager `dword_1A4D5AC` (5.10). Code reads the table pointer from the loader slot. ✅

```
mov  eax, dword_1A4D5AC          ; DataManager*
mov  ecx, [eax + 52 + 52*type]   ; DrLoaderDef[type].table  (item: +0x2080, type 159)
mov  edx, [ecx]                  ; table vtable
call [edx + 0x64]                ; Find(key_lo, key_hi)
```

Example: `World::EnterWorld` at `0x7422b8`..`0x7422dc`, item table (`+0x2080`). ✅

Tree tables do not use a slot. Code uses the special loader at DataManager `+0`..`+24` (5.10). Example: the quest table at `+0`, `IsDailyQuest` (`0xa21be0`) calls its `vtbl + 0x5C`. ✅

A slot with no loaded table holds the stub table (vtable `0x17c86c8`, 5.5 step 6). ✅ for the stub assignment; ❓ that every stub method returns null.

## 6.2 Find by key

Table `vtbl + 0x64` is `Find(u64 key)`. It returns a `DrEl*` (the record header, 3.4) or null. ✅

```
            Find(key)
               │
     ┌─────────┴───────────┐
 multi-key (kind 2)    auto-key (kind 1)
 0x622d50              0x62d3d0
     │                     │
 _useTableCache?       _useTableCache?
  yes ─► DrCacheData::Find 0x62e920      yes ─► DrCacheData::Find
  no  ─► DrMultiKeyElMap::get 0x621040   no  ─► key < elCount ? elArray[key] : null
          │
     reconstructed? ─ yes ► binary search in sorted elArray (12 B: u64 key, DrEl*)
                    └ no  ► std::map<u64, DrEl*> lookup
```

`DrCacheData::Find` (`0x62e920`, compressed tables): ✅

1. Binary search of the chunks by `startId` / `endId`.
2. `checkCache`: if the chunk has no raw data, decompress it (3.3).
3. `0x630070`: binary search of the records in the chunk by the key at `idOffset`.

Key rules: ✅ (4.6.3)

| Table kind | Key value |
| --- | --- |
| multi-key | u64 at `record + 8`. First key field in the low bits, the next fields above it. Example `item`: `id | level << 32`. |
| auto-key | position of the record in the file, from 1. Records of type `0xFF` get no key (5.9). |

## 6.3 Find by alias

Table `vtbl + 0x60` is `DrDataTableImpl::Find(const wchar_t* alias)` (`0x6307f0`). ✅

1. Format `"%s:%s"` (string `0x1533474`) with the table name and the alias. Example form: `text:` + alias.
2. Alias map (`vtbl` `0x18716d0`) `vtbl + 4` `Find` (`0x6257c0`) gives the u64 key.
3. `vtbl + 0x64` `Find(key)` (6.2).

The alias map comes from the alias block of the main file (2.5.4, 5.8 step 6). The local file has no alias block, thus local tables (for example `text`) use aliases from the main file block. ✅ for the read order; ❓ that the main block holds the `text` aliases.

Scan of `vtbl + 0x60` calls near a table load (same method as 6.5): 192 calls. Most: `arenaportal` 39, `item` 29, `text` 14, `effect` 10, `map-group-1` 10, `icontexture` 6. ✅ (heuristic count)

## 6.4 Go through all records

| Function | Address |
| --- | --- |
| `DrMultiKeyTable::createInnerIter` | `0x622e00` (helper `0x622d90`) |
| `DrMultiKeyTable::removeInnerIter` | `0x622ea0` |
| `DrAutoKeyTable::createInnerIter` | `0x62d4f0` (helper `0x62d440`) |
| `DrAutoKeyTable::removeInnerIter` | `0x62d630` |
| `DrCacheData::createInnerIter` | `0x62f570` / `0x62f5e0` |
| `DrCacheData` iterator `Next` | `0x62f430` |

Behaviour: ✅ for the addresses; ❓ for the order. The multi-key iterator walks the key map (sorted by key, ❓). The auto-key iterator walks `elArray` from key 1. The cache iterator walks the chunks in order and decompresses each chunk when it gets to it ❓.

Vtables: multi-key `0x1871588`, auto-key `0x1871748` (5.10). ✅

## 6.5 Who reads which table (reader scan)

### 6.5.1 Method

1. Take each code reference to `dword_1A4D5AC` (6,454 references in 3,736 functions). ✅
2. In the next 8 instructions, find `mov reg, [reg + N]` with `N = 52 + 52 × type`. That gives the table type. ✅
3. Add the 17 tables that keep the slot in a global (5.3.1); take the references to those globals. ✅
4. Write the demangled function names into `tables.tsv`: `readers` (first 2 names in name order and `…(+n)`, top 60 tables only; the file must stay below 40,000 bytes) and `reader_count` (all tables). ✅

Limits: ❓ The scan misses code that keeps a table pointer in a local object for a long time, and code that reaches a table through a helper. Thus `reader_count` is a low limit. 48 tables have 0 readers in the scan. That does not prove that no code reads them.

### 6.5.2 The 60 most used tables

Kind: 1 = auto-key, 2 = multi-key. Count = functions that read the table (6.5.1). The key fields are in `tables.tsv` column `key`.

| # | Type | Table | Kind | Count | | # | Type | Table | Kind | Count |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 159 | item | 2 | 1006 | | 31 | 287 | skill-training-room-subject | 2 | 32 |
| 2 | 257 | skill3 | 2 | 262 | | 32 | 66 | craft-case-recipe | 2 | 30 |
| 3 | 348 | zone | 2 | 246 | | 33 | 154 | interdungeonlist | 2 | 30 |
| 4 | 177 | itemtransformrecipe | 1 | 147 | | 34 | 219 | partychatchannel | 2 | 28 |
| 5 | 99 | effect | 2 | 144 | | 35 | 325 | tendency-field | 2 | 28 |
| 6 | 183 | job | 2 | 138 | | 36 | 276 | skillskin | 2 | 27 |
| 7 | 98 | dungeon | 2 | 98 | | 37 | 217 | npctalkmessage | 1 | 26 |
| 8 | 5 | achievement | 2 | 76 | | 38 | 268 | skill-inheritance | 2 | 26 |
| 9 | 10 | arenaportal | 1 | 62 | | 39 | 39 | boss-challenge | 2 | 25 |
| 10 | 198 | mapinfo | 2 | 59 | | 40 | 68 | craft-group-recipe | 1 | 25 |
| 11 | 262 | skillcastcondition3 | 1 | 55 | | 41 | 70 | craft-recipe | 2 | 25 |
| 12 | 113 | faction | 2 | 53 | | 42 | 92 | duel-bot-training-room | 2 | 25 |
| 13 | 138 | guild-craft-recipe | 2 | 53 | | 43 | 166 | itemgrowth | 2 | 25 |
| 14 | 243 | raid-dungeon | 2 | 51 | | 44 | 286 | skill-training-room-group | 2 | 25 |
| 15 | 349 | zoneenv2 | 1 | 51 | | 45 | 157 | item-buy-price | 1 | 24 |
| 16 | 11 | attendance | 2 | 50 | | 46 | 191 | level | 2 | 23 |
| 17 | 132 | gradebenefits | 2 | 50 | | 47 | 240 | questreward | 2 | 23 |
| 18 | 213 | npc | 1 | 50 | | 48 | 250 | rankingrewardgroup | 2 | 23 |
| 19 | 55 | content-quota | 2 | 47 | | 49 | 6 | achievement-register | 2 | 22 |
| 20 | 245 | randomstore | 2 | 47 | | 50 | 3 | account-level | 2 | 21 |
| 21 | 144 | guilduniformcomponent | 2 | 43 | | 51 | 27 | battle-royal-field | 2 | 21 |
| 22 | 200 | mapunit | 2 | 39 | | 52 | 216 | npcresponse | 1 | 21 |
| 23 | 59 | contentsjournal | 1 | 38 | | 53 | 45 | challengelist | 2 | 20 |
| 24 | 218 | party-battle-field-zone | 2 | 37 | | 54 | 50 | classic-field-zone | 2 | 20 |
| 25 | 95 | duel | 1 | 36 | | 55 | 156 | item-brand-tooltip | 2 | 20 |
| 26 | 72 | creatureappearance | 1 | 35 | | 56 | 161 | itemexchange | 1 | 20 |
| 27 | 327 | text | 1 | 35 | | 57 | 204 | mastery-level | 2 | 20 |
| 28 | 89 | duel-bot-challenge | 2 | 34 | | 58 | 267 | skillgatherrange3 | 1 | 20 |
| 29 | 142 | guildlevel | 2 | 34 | | 59 | 52 | closet-group | 2 | 19 |
| 30 | 43 | cave2 | 1 | 33 | | 60 | 53 | collecting | 2 | 19 |

Ties at 19: also `lobby-pc` (194) and `public-raid` (237). ✅ (scan result, `tables.tsv`)

## 6.6 From a server packet id to a client record

The server and the client use the **same key**. The client does not translate ids. It packs the key fields into the u64 key (4.6.3) and calls `Find` (6.2). ✅ for the three examples below; ❓ as a rule for every packet.

| Example | Address | Path |
| --- | --- | --- |
| `World::EnterWorld(G_EnterWorldOkPacket const*)` | `0x7422a3`..`0x7422dc` | Reads 4 int32 item ids from the packet data (base `+0x34320`). Skips a value ≤ 0. Key = id in the low 32 bits, level byte = 1 in the high 32 bits. Calls item `Find`. |
| `ItemPouch::checkSendBoastEventMsg(G_PickupItemOkPacket const*)` | `0xd0a894`..`0xd0a8d9` | Reads a pouch slot index (u8 at `+8` of a packet entry; `0xFF` = none). `ItemPouch::GetPouchItemInfoByItemIndex` gives a `PouchItemInfo`. Its u64 at `+8` is the item key. Calls item `Find`. |
| `BItem::Init` | `0xd95842`..`0xd95861` | The item property object gives the u64 key (its `vtbl + 0x34`). Calls item `Find`. |

```
server packet ──► int32 id (+ level / variation / step)
                    │  pack (4.6.3): low 32 = first key field, high = next fields
                    ▼
              u64 key ──► DataManager slot[type].table ──► vtbl+0x64 Find ──► DrEl* record
```

Rules for a server port (from 6.2 and 4.6.3):

1. Multi-key tables: send the same key field values that are in the record (`item`: `id`, `level`; `skill3`: `id`, `variation-id`). ✅ key fields from the schema.
2. Auto-key tables: the key is the record position (6.2). It changes when the file changes. ❓ if the server sends these keys; for named records the client uses the alias (6.3).
3. A key that is not in the table gives null. Each caller tests for null (seen in `0x7422b4`) ❓ for all callers.

## 6.7 Notes for a Rust port

- One table lookup = one map from u64 key to record. A multi-key table needs a sorted map; an auto-key table needs a vector index from 1.
- An alias lookup = a map from `"table:alias"` to u64 key.
- The existing Rust reader is in `bnsdata` (3.11). This chapter did not run `tools/rust_api.py` (no Bash in this session) ❓ for the exact Rust names.

## 6.8 Open questions (❓)

1. Stub table methods (6.1).
2. Iterator order (6.4).
3. Readers that the scan misses (6.5.1).
4. Auto-key ids in server packets (6.6).

## Journal (old documents, not changed)

- `Docs/Client/client-datafiles.md`
- `Docs/DataFile/06-client-body.md`, `08-strings.md`, `STATUS.md`
