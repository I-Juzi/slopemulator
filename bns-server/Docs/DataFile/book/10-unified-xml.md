# 10. One XML source for client and server data

This chapter defines one merged XML data set. It holds the client tables and the server tables together. A tool (`bnsxml`, card `Docs/Plan/cards/dfx-4-rust.md`) dumps both sides, merges them, and builds both sides again from the merged set. One edit then changes the client and the server, and the two sides stay in sync (8.6).

Build: `Client.exe` 0.0.381 x86, IDA copy `ida/lanes/cx/Client.exe.i64`. Server data: `GameData-0.0.5002.50`, `GameDaemon-0.0.356.8492` (8.1). All addresses are in the 381 copy. ✅ = verified in this copy on 2026-11-10. ❓ = not verified.

## Quick find

| Question | Section | Address or file |
| --- | --- | --- |
| Why one source? What is the data flow? | 10.1 | 8.4.2 |
| Which XML element and attribute names does the client read? | 10.2.1, 10.2.2 | `0x61ad20`, `0x61b0e0`, `0x6199d0` |
| How must a value be written as text (int, bool, enum, ref)? | 10.2.3 | `0x61cf40`, `0x61b660` |
| Does the client read `release-side`? | 10.2.4 | string search (no) |
| What does the merged file look like? | 10.2.5 | — |
| Original bugs that the port must keep | 10.2.6 | `quirk_*` |
| How do I match a client table and a server table? | 10.3.1 | `tables.tsv`; root `type`, `version` |
| Field classes and which side wins | 10.3.2 | [merge-rules.tsv](merge-rules.tsv) |
| Worked examples `terrain`, `achievement`, `item` | 10.3.3 | `tables.tsv`; `02-tables.md` |
| Which fields are derived (hidden)? Who fills them? | 10.4 | `0x5bcd50`, `0x5bcca0`, loaders `0x5bce90`–`0x5c83e0` |
| How do I build the client and the server files again? | 10.5 | 2.7, 9.2 |
| What proves a round trip? | 10.5.3 | — |
| Open questions | 10.6 | — |

## 10.1 Why one source

The game company made the client files and the server files from one designer XML source ❓ (8.4.2; inference from the names). Each side got only its own fields. We do not have that source. We can make a source again: dump the client tables to XML, dump the server tables to XML, and merge them.

```text
  datafile*.bin / localfile*.bin           GameDaemon/data/*.xml (LMXBOSLB)
        |  dump-client (10.2 form)                 |  dump-server (lmxb)
        v                                          v
  client/<table>.xml  ------------+  +-----  server/<table>.xml
                                  v  v
                          merge (10.3 rules)
                                  |
            merged/<table>.xml  +  merge-report.tsv  +  fields.tsv
                                  |
                 +----------------+----------------+
                 v                                 v
        build-client (10.5.1)             build-server (10.5.2)
   datafile*.bin / localfile*.bin      GameDaemon/data/*.xml (LMXBOSLB)
                 \______ verify (10.5.3): dump again, compare ______/
```

The client can read a table from XML (5.6). Thus the 10.2 form is not only a tool format. It is the form that the client XML loader accepts.

## 10.2 The XML form that the client accepts

The client XML loader is `DrElReaderImpl` (PDB name). It uses the same `DrTableDef` definitions as the binary loader (4.1). The XML parser is in `XmlReader_cl32.dll` (5.2). That DLL is not in our IDA copy ❓. This section gives what the client code asks from the parser.

### 10.2.1 Elements

| Item | Rule | Source |
| --- | --- | --- |
| Element name → element index | The reader compares the element name with `DrElDef.name` of each element of the table, case-insensitive (`_wcsicmp`). No match → index −1 (the element is not read). Element 0 is `table`, element 1 is `record` (4.3.2). | `findEltype` `0x61bdf0` ✅ |
| Record element | Reads the subtype, then all attributes of the element body and of the subtype (`sub_61AFD0` → `sub_61B220` per attribute). | `sub_619d40` ✅ |
| Subtype | The reader reads the attribute `type` (a static `DrAttrDef` at `0x13f3cf0`, name `type`, type 7 string). If the element body has subtypes (`subcount` > 0), it finds the text in the `DrSubDef` names. Not found or empty → the element fails. No subtypes → `subtype` = −1 and `type` is not read. | `initEl` `0x61ad20`, `findSubtype` `0x61bd90` ✅ |
| Record size | `DrEl.size` = body size, or `sub[s].size` when a subtype is set. | `0x61ad20` ✅ |
| Alias | The reader finds the attribute named `alias` in the record body (`sub_61BE40`, case-insensitive). 217 of 362 tables have it ✅. Text up to 70 characters (buffer 142 bytes). | `sub_6199D0` (`ReadAlias`, vtable call from `DrMultiKeyTable::LoadAlias` `0x620ba0`) ✅ |
| Alias key | Multi-key table: the key comes from the key attributes of the record (4.6.3). Auto-key table: the key is the record counter that the caller gives. | `0x6199d0` ✅ |
| Alias map entry | The string `<table name>:<alias>` (format `%s:%s` at `0x1533474`) → 8-byte key. An empty alias is accepted only if the `alias` attribute has `required` = 0. A deprecated `alias` attribute is skipped. | `0x6199d0` ✅ |

`LoadAlias` runs over all documents of the loader slot (`xmldoc.doccount`) and over all child elements of each root. Thus one table can come from many files (the `item` glob has six patterns, 8.4.2). ✅ `0x620ba0`.

### 10.2.2 Attributes

The reader asks the parser for one attribute **by name** (parser vtable slot `+24`, with the `DrNameHash` hash and name). It does not walk the attributes of the element. ✅ `sub_61B0E0` `0x61b0e0`, `sub_61B220` `0x61b220`.

| Rule | Behaviour | Source |
| --- | --- | --- |
| Name | `DrAttrDef.name` (4.3.4). | ✅ |
| Repeated field (`repeat` > 1) | Repeat `i` (0-based) has its own name `<name>-<i+1>`, from the `DrNameHash` array. Example: `item.filter-equip` repeat 4 → `filter-equip-1` … `filter-equip-4`. All 13,544 repeat names in 381 follow this rule, 0 exceptions. | ✅ IDA scan of all definitions |
| Type 26 `native` | The value is the text of the element, not an attribute. | ✅ `0x61b0e0` |
| Deprecated attribute | Not read. The reader returns at once. | ✅ `0x61b0e0` |
| Missing, key attribute | The record fails. | ✅ |
| Missing, `i` < `required` byte | The reader stops this field and returns success. The value stays as the default record gives it. See `quirk_required_stops_repeats`. | ✅ |
| Missing, other | `sub_61B0E0`: use `defvalue` text if it is set, else keep the default. `sub_61B220` (record path): use `defvalue` only for `string`, `ref`, `tref`; other types keep the default record value. | ✅ |
| Value does not parse | Key attribute → the record fails. `i` < `required` → stop, success. Else → go to the next repeat. | ✅ |
| Unknown attribute in the file | Not read (the reader asks by name only). Thus extra attributes are safe for the client. | inference from by-name lookup ✅; an XSD check in the DLL ❓ (`DrLoaderDef.xsd`, 5.3.2) |

### 10.2.3 Value text per type

Dispatch on `DrAttrDef.type` in `sub_61CF40` (`0x61cf40`) ✅. Type codes: 4.6.1.

| Type (code) | Text form | Source |
| --- | --- | --- |
| int8 (1), int16 (2), int32 (3), int64 (4), su (13), distance (21), velocity (22) | Decimal digits (`isNumber`, then `_wtoi64`). The value must be in `min` … `max` of the definition. | `sub_61B430` `0x61b430` ✅; signed/unsigned compare ❓ (`quirk_int_range_unsigned`) |
| float32 (5) | Decimal float ❓ (`isFloat` `0x61bc20`; parser not traced) | ❓ |
| bool (6) | `y` → 1 (checked first). Then `yes`, `t`, `true` → 1; `n`, `no`, `f`, `false` → 0. All case-insensitive. Other text → parse error. | `0x61cf40`, `sub_61B510` `0x61b510` ✅ |
| string (7) | Text as is. | `__readStr` `0x61b5a0` ✅ |
| seq (8), prop_seq (23) | One name of the `seq` list, case-insensitive. Stored value = index in the list (u8). No match → parse error. | `sub_61C690` `0x61c690` ✅ |
| seq16 (9), prop_field (24) | The same, stored as i16. | `sub_61C6F0` `0x61c6f0` ✅ |
| ref (10) | See the ref table below. | `readRef` `0x61c750` → `__readRef` `0x61b660` ✅ |
| tref (11) | A typed ref (table type u16 at +0, key at +4) ❓ text form (`sub_61C780`) | ❓ |
| msec (20) | Integer; must be a multiple of the global divisor at `0x1949470` (value 1 in 381). | `sub_61C5F0` ✅ |
| vector16/32 (14, 15), icolor (16), fcolor (17), box (18) | Several numbers in one text ❓ (`sub_61C080`, `sub_61C1B0`, `sub_61C2D0`, `sub_61C400`, `readBox`) | ❓ |
| script_obj (25), version (27), icon (28), time32/time64 (29, 30) | ❓ (`sub_61C8D0`, `sub_61CCA0`, `sub_61CD40`, `sub_61CEC0`) | ❓ |

**Ref text (`__readRef`, `0x61b660`)** ✅. The reader first removes blanks (`shinkBlank`).

| Text | Result |
| --- | --- |
| empty, or `null` (case-insensitive) | key 0 (no target) |
| digits only (`isKey`) | The number is the 8-byte key. Accepted only if the target table is multi-key (`autokey` = 0). |
| digits with dots, for example `1234.2` (`isMultiKey`: no leading or trailing dot, at least one dot) | One part per key attribute of the target record element, in definition order, separated by `.` (`tokenize`, 70 characters for each part). Each part is read with the rule of its type. The key is the 8 bytes at `keyoffset` of the built record (4.6.3). Target must be multi-key. |
| other text | Alias. The reader looks up `<target table name>:<text>` in the alias map. Not found → parse error. |

Thus a ref can name its target three ways. The merged form uses the alias when the target has one, else the dotted key (10.2.5).

### 10.2.4 Server LMXBOSLB form compared

| Item | Client XML reader | Server LMXBOSLB files | Same? |
| --- | --- | --- | --- |
| Root element | element 0 `table` (4.3.2) | root element with `release-module`, `release-side`, `type`, `version` (8.2.3) | name ✅ same; the root name is in the file ❓ (`02-tables.md` "Other binary root names": `combat-sequence-root`) |
| Record element | `record` | direct children of `table` named `record` (`02-tables.md` "Records are direct element children of `table`") | ✅ |
| Repeat names | `<name>-<n>` | `<name>-<n>`, for example `item-skill-1` … `item-skill-16`, `register-ref-1` … `register-ref-5` (`02-tables.md` `item`, `achievement`) | ✅ |
| Bool | `y`/`n` and others | `y`/`n` (for example `item.account-used`) | ✅ |
| Ref | alias, number or dotted key | text, mostly aliases (`02-tables.md` "Reference targets") | ✅ alias form; other forms in server data ❓ |
| `release-side`, `release-module` | The client exe has no UTF-16 string `release-side` or `release-module` ✅ (byte search). The client does not read them. The XML DLL can ❓. | always `server` in GameDaemon | — |
| `type`, `version` on the root | not read by the code above ❓ (the loader slot gives the table; the file glob gives the file) | table name, `major.minor` | — |

Result: one record element form can serve both sides. The root attributes are a server need. The merged form keeps them.

### 10.2.5 The merged file form

One file for each table: `merged/<table>.xml`, UTF-8. The `<table>` name is the client table name (lower case, `tables.tsv`). Example (the values are a form example, not game data):

```xml
<?xml version="1.0" encoding="utf-8"?>
<table type="terrain" version="0.8"
       client-version="0.8" server-version="0.8"
       release-module="ZoneData1" _server-type="Terrain">
  <record id="1" alias="Terrain_Example" umap-name="Example_P"
          terrain-start-x="0" terrain-start-y="0" count-x="4" count-y="4"
          terrain-origin-x="0" terrain-origin-y="0" description="example"
          _side="both" _server-file="terraindata.3040.xml"/>
</table>
```

| Rule | Why |
| --- | --- |
| Attribute names = client names (10.2.2), repeats as `<name>-<n>` | The client reads this form; the server uses the same names (10.2.4). |
| Values in the 10.2.3 text form. Bool as `y`/`n`. Enum as the `seq` name in the case of the definition. Ref as alias, else dotted key, else `null` for key 0. | One text form for each value makes compare exact. |
| Names that start with `_` are tool data (`_side`, `_server-file`, `_server-type`, `_line`, `_source`). The builds do not write them. | The client ignores unknown attributes (10.2.2) ❓ XSD. No 381 attribute name starts with `_` ❓ (to check in `merge`). |
| `client-version`, `server-version` on the root. `version` = the client version. | Version gap report (10.3.1). |
| Derived (hidden) fields are not written (10.4). | The build makes them again. |
| Server attribute order and `_line` (LMXBOSLB `source_line`) are kept for server records | Needed for a byte-equal server build (10.5.2). |

### 10.2.6 Quirks of the client XML reader

| Name | Address | What the code does | Value it gets | Who can reach it |
| --- | --- | --- | --- | --- |
| `quirk_required_stops_repeats` | `sub_61B0E0` `0x61b0e0`, `sub_61B220` `0x61b220` | When repeat `i` of a field is missing (or does not parse) and `i` < the `required` byte (+11), the reader returns success and does not read the later repeats. | The later repeats keep the default record value, also if the file has them. Example: `x-1` missing, `x-2` present, `required` = 1 → `x-2` is not read. | Any merged or dumped XML with a gap in a repeat list. `dump-client` must write all repeats up to the last non-default one. |
| `quirk_bool_strict_text` | `sub_61B510` `0x61b510` | Only the eight words in 10.2.3 parse. `1` and `0` do not parse. | Parse error; not a key → the next repeat; the field keeps the default. | Server data that writes bools as numbers (`02-tables.md`: "Numeric 0/1 stays int") ❓ which fields. |
| `quirk_int_range_unsigned` ❓ | `sub_61B430` `0x61b430` | The decompiler shows an unsigned 64-bit compare with `min`/`max`. | A negative value can fail the range check ❓. | Negative values in fields with `min` < 0 ❓. Check with a test in a later slice. |
| `quirk_ref_number_multikey_only` | `__readRef` `0x61b660` | A number or dotted key is accepted only when the target table is multi-key. | For an auto-key target, a number is a parse error. | Refs to auto-key tables (119 tables, 4.6.2) must use the alias. |
| `quirk_hidden_first_writer` | loaders in 10.4.2 | A loader writes a hidden ref only when the target field is still empty. | The first loader in call order wins. | `zone.attraction` when one zone is in two attraction tables. |
| `quirk_hidden_last_writer` | `loadHiddenFieldFromQuest` `0x5c83e0` | Walks quest IDs from 1 upward and writes `dungeon.quest` each time. | The highest quest ID that names the dungeon wins. | Two quests with the same dungeon. |
| `quirk_tooltip_slots_full` | `loadHiddenFieldFromSkillTooltip` `0x5bda70` | Appends into the first free slot; stops at 5 slots (10 for `sub-tooltip`). | Later tooltips are not linked. ❓ exact behaviour on overflow (skip or stop) | `skilltooltip` with more than 5 rows for one skill and group. |

## 10.3 Field mapping rules

The full rule list, one row for each class, is in [merge-rules.tsv](merge-rules.tsv). This section gives the reasons.

### 10.3.1 Match tables: by name and version, never by ID

| Step | Rule | Source |
| --- | --- | --- |
| 1 | Client table name = `DrTableDef.name` (`tables.tsv` `name`). Server table name = root attribute `type`. | 4.3.1; 8.2.3 |
| 2 | Compare names case-insensitive, and also with `-` removed. Example: server `Terrain` = client `terrain`; server `ZoneEnv2` = client `zoneenv2`; server `FielditemDrop` = client `fielditemdrop`. | `02-tables.md` table list; `tables.tsv` |
| 3 | Never match by table ID. The IDs differ between builds (7.9, 4.7). | 4.7 |
| 4 | Compare the versions (`major.minor`). Equal → field merge (10.3.2). Different → class `version-gap`. | 8.4.1 |
| 5 | A name with no partner → `client-only-table` or `server-only-table`. | — |

`version-gap` rule: the merge does not merge fields of the two sides. It keeps two record sets in the merged file: `<record _side="client" …>` and `<record _side="server" …>`. Each build takes only its own set. The report row has class `version-gap` and both versions. Reason: a different version means a different field list (8.6), and a value that has the same name can have a new meaning ❓. The operator must merge these tables by hand, or get the matching data build. This keeps the gap visible (card: "report this and not hide it").

Known versions (8.4.1): `achievement` 0.19 = 0.19, `terrain` 0.8 = 0.8, `item` 1.398 (client) vs 1.387 (server 348 definition; the 2020 data value ❓). `merge` reads the server value from the root `version` and decides.

### 10.3.2 Match records and fields; classes

Records: match by the **key**, not by the record position.

| Table kind | Key of a client record | Key of a server record |
| --- | --- | --- |
| multi-key (kind 2) | the key fields of the definition (`tables.tsv` `key`), as text in 10.2.3 form, joined with `.` (the dotted key of 10.2.3) | the same attributes in the server record |
| auto-key (kind 1) | no key fields; use `alias` | `alias` |
| auto-key with no `alias` attribute | — | no match possible → each record is `client-only-record` or `server-only-record` ❓ (count in the report) |

Fields: match by attribute name (10.2.2). Compare the **normalized text** (10.2.5): bool to `y`/`n`; enum to the `seq` name of the client definition; numbers without leading zeros; refs to the target key (alias resolved with the alias map of the side that wrote it).

| Class | Test | Merged value | Why |
| --- | --- | --- | --- |
| `both-equal` | name in both; normalized texts equal | that value | No choice. |
| `both-conflict` | name in both; texts differ | **server** value (default); `--prefer client` changes it. Both texts go to `conflicts.tsv`. | The server enforces the rule (price, rate, count). The client only shows it. A client that shows the server value is correct. The other way is not. |
| `client-only` | attribute in the client definition, not in the server record | client value | The server does not get this field (text refs, icons, UI data). |
| `server-only` | attribute in the server record, not in the client definition | server value | The client does not get this field. |
| `derived` | client attribute with `hidden` = 1 | not written | The build makes it (10.4). |
| `deprecated` | client attribute with `deprecated` = 1 (size 0, 4.3.4) | server value if the server has it, else not written | The client binary has no bytes for it (4.3.4). |
| `client-only-record` | key only in the client dump | the client record, `_side="client"` | Client-only rows (for example display rows). Report the count. |
| `server-only-record` | key only in the server dump | the server record, `_side="server"` | Server-only rows. Report the count. A shared value in such a row cannot be shown by the client (8.6). |
| `version-gap` | table versions differ | both record sets, no field merge | 10.3.1. |
| `client-only-table`, `server-only-table` | name has no partner | the records of that side | — |

Key and `alias` fields can not be in `both-conflict`. They define the match. An `alias` that differs on a matched key is a `both-conflict` row with field `alias` and is reported. The server alias wins, because server scripts use aliases (`Table::by_alias`, 8.6).

### 10.3.3 Worked examples

The client fields come from the 381 definitions (IDA, 2026-11-10). The server attributes come from `Docs/Server/GameDaemon/02-tables.md`. Notation: `K` key, `H` hidden, `xN` repeat.

**`terrain` (client 0.8, server `Terrain` 0.8, 164 records).** Client: `id`K int16, `alias`, `umap-name`, `terrain-start-x`, `terrain-start-y`, `count-x`, `count-y`, `terrain-origin-x`, `terrain-origin-y`, `description` (10 fields, `tables.tsv`). Server: the same 10 names. Expected report: 0 `client-only`, 0 `server-only`, 10 shared fields; record classes from the key `id`. This is the best first test table.

**`achievement` (client 0.19, server 0.19, 1,558 server records).** Key `id`K int16, `step`K int16, `job`K prop_seq (4.6.3).

| Class | Fields |
| --- | --- |
| shared (`both-*`) | `alias`, `id`, `step`, `job`, `deprecated`, `complete-from-client`, `register-ref-1..5`, `register-type-1..5`, `register-value-1..5`, `step-complete-reward-item-1..2`, `step-complete-reward-item-count-1..2`, `step-complete-reward-game-cash`, `step-complete-reward-skill-build-up-point`, `current-step-score`, `completed-effect` |
| `client-only` | `progress-show`, `ability`, `ability-value`, `completed-effect-category`, `completed-effect-order`, `title-*`, `category-1`, `category-2`, `map-group-1`, `icon`, `name2`, `description2`, `sort-no`, `completed-game-message`, `talk-social` (UI and text refs) |
| `derived` | `step-complete-reward-type`, `step-complete-reward-final-item-1..2`, `step-complete-reward-final-item-count-1..2` (`hidden` = 1). The server XML does not have them ✅ (`02-tables.md`). |
| `server-only` | none seen in the `02-tables.md` list (the list there ends at `step-complete-reward-skill-build-up-point`) ❓ full list |

Note: `achievement` has a real field named `deprecated` (bool). It is not the `DrAttrDef.deprecated` flag. The server writes `deprecated="y"` in 72 records ✅ (`02-tables.md`).

**`item` (client 1.398, server ❓ 1.387; 144,241 server records in 14 files).** Key `id`K int32, `level`K int8. 307 base fields, 6 subtypes with 177 more fields (`tables.tsv`). 6 base fields are hidden; with the repeat lists, `forwarding-types`, `quests`, `missions`, `cases`, `case-subtypes` (x64 each), `awakening-target-item` (x5) and `skill-book-achievement-*` are hidden (IDA scan).

| Class | Example fields |
| --- | --- |
| shared | `id`, `level`, `alias`, `price`, `acquire-used`, `account-used` |
| `server-only` (not in the client **base** fields) | `attach-equip-gem-piece-*`, `ability-feeding-*` ✅ base; the 177 subtype fields not checked ❓ |
| `derived` | `quests-1..64`, `forwarding-types-1..64`, `awakening-target-item-1..5` |
| table class | `version-gap` if the 2020 root `version` is not 1.398 ❓ |

The 14 server files of `item` must stay 14 files. `_server-file` on each record keeps the file name (10.2.5).

## 10.4 Derived and hidden fields

### 10.4.1 When the client fills them

| Fact | Source |
| --- | --- |
| 281 attributes have `hidden` = 1, in 49 tables (base records, subtypes and tree elements). The 143 of 4.6.2 count base records only. | IDA scan of all definitions ✅ |
| The 381 client has **38** loader functions `loadHiddenFieldFrom*` / `LoadHiddenFieldFrom*`. The number 52 in 8.4.3 also counted 14 STL helper functions whose names contain the text (`AwakenTargetItem` map). | IDA function names ✅ |
| 34 loaders run only in `constructDataOnlyForXML` (`0x5bcd50`). Each one first checks that its source table type is in the set of tables loaded from XML (`std::set::find`). | callers ✅ |
| `Load` (`0x5bc500`) calls `constructDataOnlyForXML` only when `loadBinaryDataFile` failed. | `0x5bc500` ✅ |
| 4 loaders run each time, from `constructData` (`0x5bcca0`): `DungeonQuestRelated` `0x5bee40`, `TencentQuest` `0x5c2b80`, `MasteryLevel` `0x5c2cb0`, `Quest` `0x5c83e0`. They use the tree table `quest`, which is never in the binary file (4.5). | callers ✅ |

Consequence: when the client loads `datafile.bin`, the 34 loaders do not run. The hidden values must already be in the binary records. Thus the original packer computed them ❓ (the packer is not in our files, 1.3). A `build-client` that writes 0 into hidden fields gives a client with empty links (for example no zone attraction). **Rule for the merged XML:** a derived field is not stored. `build-client` computes it with the rules of 10.4.2 and writes it into the binary record. `dump-client` reads it and `verify` compares the computed value with the dumped value.

Many PDB names of the loaders are ported from GM 419 and do not agree with the code (5 intro). The table below names the tables that the code uses.

### 10.4.2 Rules (verified)

| # | Loader (address) | Source table → target field | Rule |
| --- | --- | --- | --- |
| 1 | `loadHiddenFieldFromFieldZone` `0x5bdfb0` | `field-zone.zone-1..30` (ref → `zone`) → `zone.attraction` (tref, offset 44) | For each `field-zone` record and each non-zero `zone-n`: find the `zone` record. If its `attraction` table type word is 0, set `attraction` = (type 118 `field-zone`, key of the `field-zone` record). `quirk_hidden_first_writer`. ✅ |
| 2 | 17 more loaders with the same form (`Cave2`, `ClassicFieldZone` (type 50), `FactionBattleFieldZone`, `GuildBattleFieldZone`, `JackpotBossZone`, `RaidDungeon`, `BossChallenge`, `PartyBattleFieldZone`, `DuelBotChallenge`, `DuelBotTrainingRoom`, `TendencyField`, `SkillTrainingRoom`, `PublicRaid`, `BattleRoyal`, `ChallengeParty`, `TimeLimitField`, `ContentsReset`) | the named attraction table → `zone.attraction` | Same rule, with the type of the source table. All write offset 44 of `zone` ✅ (store scan); the zone list field of each source table ❓. `RaidDungeon` and `SkillTrainingRoomGroup` also write `zone.dungeon-step` ❓ rule. |
| 3 | `loadHiddenFieldFromSkillTooltip` `0x5bda70` | `skilltooltip.skill` (ref → `skill3`), `skilltooltip.tooltip-group` (seq `M1`, `M2`, `SUB`, `STANCE`, `CONDITION`) → `skill3.main-tooltip-1`, `main-tooltip-2`, `sub-tooltip`, `stance-tooltip`, `condition-tooltip` | For each `skilltooltip` record: find the `skill3` record; append the `skilltooltip` key into the first empty slot of the list that `tooltip-group` selects (5 slots; 10 for `sub-tooltip`). `quirk_tooltip_slots_full`. ✅ |
| 4 | `loadHiddenFieldFromSkill3` `0x5bd020` | `skillcastcondition3`, `skill3` → `skill3.combo-start-point-flag`, `skill3.use-skill-message`, `skill3.ui-next-combo`, `skillcastcondition3.last-event-result` | Writes these fields only on records with `DrEl` byte +2 = 0 ❓ (meaning of the byte). `combo-start-point-flag` := 0 and `use-skill-message` := 1 in two branches ✅; the conditions ❓. |
| 5 | `loadHiddenFieldFromQuest` `0x5c83e0` (always runs) | `quest` (tree table) field at quest +96 (a `dungeon` ref) → `dungeon.quest` (offset 808) | For quest ID 1 … max: if the quest has a dungeon ref, set `dungeon.quest` = quest ID, and copy a byte from the dungeon (+20) into the quest (+104). `quirk_hidden_last_writer`. ✅ Also writes `attraction-quest` of `classic-field-zone`, `faction-battle-field-zone`, `jackpot-boss-zone`, `boss-challenge` and `map-group-1.talk-to-self-quests*` ✅ (store scan); rules ❓. |
| 6 | `loadHiddenFieldFromChallengeList` `0x5c34e0` | `achievement` / `achievement-register` → `skill-training-room-group.achievement` (offset 112) | Writes the key of an achievement record ✅ (store); the match rule ❓. |

Store scan = a script that maps the `[reg + offset]` stores of each loader to the hidden attributes of the tables that the loader uses (`*RecordPtr::GetNSetTable` statics). A match by offset only can be a false match; rows 1, 3 and 5 are also read in the decompiler.

### 10.4.3 What the tools do with derived fields

| Tool step | Behaviour |
| --- | --- |
| `dump-client` | Writes hidden fields to a side file `derived/<table>.tsv` (key, field, value), not into the record. |
| `merge` | Class `derived`. Not in the merged XML. |
| `build-client` (next card) | Runs the rules of 10.4.2 for each target field that has a rule; for a hidden field with no rule, copies the dumped value from `derived/` by key ❓ (a fallback, report it). |
| `build-server` | Does not write derived fields. The server XML did not have them (`achievement`, 10.3.3). |

## 10.5 Rebuild paths

```
 merged/<table>.xml ──build-client──► TableBody ──Container::write──► datafile*.bin ──(re-sign ✗)──► .dat
        │                               ▲ derived/ (10.4)
        └──────────build-server──► lmxb::Document ──lmxb::encode──► LMXBOSLB file (_server-file, _line)
```

### 10.5.1 Client

| Step | Exists now | Missing | Source |
| --- | --- | --- | --- |
| Read the table list and bodies of `datafile*.bin` / `localfile*.bin` | `bnsdata::dat::Container::parse`, `bnsdata::table::TableBody::parse` | — | crate `BnsData` (`pub` API) |
| Change one field of an existing record | `bnsdata::scalar::set_scalar`, `bnsdata::text::RecordEditor` | — | crate `BnsData` |
| Write the bodies back into the container | `TableBody::write`, `Container::replace_tables`, `Container::write`; `bnsmodel::Project::pack` | — | crates `BnsData`, `BnsModel` |
| Make a record from field text (new record, other field order) | — | A record encoder: `TableDefinition` + field text → record bytes, with the string pool. The XML loader does this in the client (`0x61cf40` per field); the port must do the same. | 10.2.3 |
| Fill derived fields | — | The rules of 10.4.2; fallback `derived/` | 10.4 |
| Sort records by key | — | Key order of the binary table ❓ (sorted or file order) | 5.x ❓ |
| Write the `.dat` archive | — | AES and the RSA signature (2.4). A changed archive needs a new signature (`keyhere`). The private client loads `binreplace` files instead (2.7). | 2.4, 2.7, 9.2 |

Result: `build-client` writes `datafile*.bin` / `localfile*.bin` for the `binreplace` path only. A signed `.dat` is out of scope.

### 10.5.2 Server

| Step | Exists now | Missing | Source |
| --- | --- | --- | --- |
| Read an LMXBOSLB file | `lmxb::decode` | — | crate `Lmxb` |
| Write an LMXBOSLB file | `lmxb::encode(&Document)` | — | crate `Lmxb` |
| Make a `Document` from the merged XML | — | Group records by `_server-file`; restore the root attributes (10.2.5); remove the `_` attributes; set `source_line` from `_line`; keep the attribute order of the dump. | 10.2.5 |
| New records (no `_server-file`) | — | Put them in the file of the table with the highest file name ❓ (a tool rule, not a game rule). | — |
| Fields of class `client-only` | — | Do not write them to the server file. The server definition does not have them (10.3.2). | merge-rules.tsv |

The GameDaemon reads attributes by name (8.x). The attribute order and `source_line` do not change behaviour ❓, but they are necessary for a byte-equal file.

### 10.5.3 Round-trip checks

| # | Check | Pass rule |
| --- | --- | --- |
| R1 | Client: dump → merge (empty server side) → build-client, no edit | Each unchanged table: body bytes equal to the original (reuse the original body, as `Project::pack` does). |
| R2 | Client: same, one field edited | Only the edited record differs; `bnsmodel::Project::record` of the new file gives the same `FieldView` values for all other records. |
| R3 | Client derived fields | For each hidden field with a rule in 10.4.2: computed value = dumped value from `derived/`. Report each difference. |
| R4 | Server: dump → merge (empty client side) → build-server, no edit | Each file: `lmxb::encode` output byte-equal to the input. If not: `lmxb::element_count` equal and each element has the same attributes in the same order. |
| R5 | Merge stability | merge of the merged output with itself gives `fields_conflict` = 0, and the same `merge-report.tsv` counts. |
| R6 | Full merge | `merge-report.tsv` counts equal over two runs (no order effect). `fields_server_only` = 0 for `terrain` (10.3.3). |
| R7 | Game check (manual) | Client starts with the `binreplace` files and the server starts with the built files; no table load error in the logs. ❓ |

## 10.6 Open questions

| # | Question | Where to look |
| --- | --- | --- |
| Q1 | The int range check: is the compare signed or unsigned? (`quirk_int_range_unsigned`) | `sub_61B430` |
| Q2 | Zone list field of each of the 17 attraction loaders (10.4.2 row 2). | loaders `0x5bce90`–`0x5c83e0` |
| Q3 | Conditions of the `skill3` hidden writes (`DrEl` byte +2). | `0x5bd020` |
| Q4 | Match rule of `skill-training-room-group.achievement`. | `0x5c34e0` |
| Q5 | Are the records of a binary table sorted by key? | 5.x, table writer |
| Q6 | Is the XML parser in `XmlReader_cl32.dll` strict about entities and encoding? Not in the IDA copy. | `XmlReader_cl32.dll` |
| Q7 | Does the GameDaemon depend on attribute order or `source_line`? | 8.x |
