# 09 — Items, inventory and equipment (Client.exe 0.0.381)

1. The client keeps items in one `Inventory` object at `BPlayer+0xB50` (ctor `0xe7d290`). It has one slot vector for each depot (bag, equip, warehouse, wardrobe, …).
2. Each item is a `BItem` that holds a property sync object. There are five item tables: Weapon, Costume, Grocery, Gem and Accessory (§1.3).
3. The server sends all item changes (add, remove, stack, move, equip) in one packet: cmd 1332 `InventoryItemListOk` with an `itemReason` (§2). There is no separate add, remove or stack packet.
4. Each C2S item request sets a lock (`Inventory+0x1E5`). The handler `0xed3820` releases the lock when the reply comes (§1.5).
5. The wardrobe (closet) uses `ClosetGroup` packets 934–939. The NPC warehouse is depot 3 and uses `MoveItemReq`. The account goods warehouse uses `GoodsInventory` packets 802–808 and 1411–1412 (§5, §6).

```
 C2S request (Inventory::Request*, ItemStorePanel, ClosetGroup)
   |  checks lock Inventory+0x1E5, sets +0x1E8 = cmd
   v
 BNSClient::SendToGamed ----> server
                                 |
 S2C 1332 InventoryItemListOk <--+   (or xxxFail / xxxOk)
   v
 World::ProcessNetObjectEvent 0x749550
   v
 BPlayer::ProcessNetObjectEvent 0xbbfe10
   v
 Inventory vtable slot 3 = 0xed3820  -- reset lock for listed cmds
   |-- for each element: GS_ItemProperty variant -> new BItem -> BItem::Init
   |-- 0xe616a0  set slot (item, depot, pos)   / data_id==0 -> remove
   |-- itemReason 1,26,57,58,60,68 -> OnNotifyItemAcquired + message
   v
 Panels: NewInventoryPanel2 0xe2faa0, InventoryEquipPanel 0xdfb3a0,
         WardrobePanel 0x7b8110, WarehousePanel 0x791320, ItemStorePanel 0xd6cc00
```

Sources: Client.exe 0.0.381 IDA database (all addresses are 381). Enum names and values come from the 419 GM database and are marked "419 GM". Packet ids and names come from `game-commands.tsv` (see chapter 05).

Related chapters: [05 — game dispatch](05-game-dispatch.md) (packet framing, `StructPacket::Def`, T_SUB), [06 — movement](06-movement.md), [08 — creature](08-creature.md) (`T_PROP_SYNC` wire format, `CreatureProperty`). This chapter does not repeat them.

## Files

| File | Content |
| --- | --- |
| `09-items-commands.tsv` | 262 item commands: id, name, direction, 381 handler, route, struct size, 381 builder, lock reset flag, fields. |
| `09-items-properties.tsv` | 147 property fields of the five item tables, with 381 FIELD_INFO address, offset and sync offset. |
| `09-items-enums-gm.tsv` | 480 item enums in full (419 GM), 4,112 rows. |
| `09-items-senders.tsv` | C2S senders found by `mov word ptr [reg+4], cmd`. |
| `09-items-functions.tsv` | 6,071 item and inventory functions with class, size and first string. |

## Quick find

| Task | Function | 381 address | GM name | Section |
| --- | --- | --- | --- | --- |
| Item change packet handler | (no name) Inventory event handler | `0xed3820` | Inventory::ProcessNetObjectEvent ❓ | §2.1 |
| 1332 packet decoder | handler 1332 | `0xef1430` | G_InventoryItemListOk | §2.1 |
| Set or clear one slot | false name `CommonCommandAutoComplete::setCandidateListPcItemDataID` | `0xe616a0` | Inventory set slot ❓ | §1.4 |
| Get item at depot/pos | `Inventory::GetItem` | `0xe3dd40` | Inventory::GetItem | §1.4 |
| Move item | `Inventory::RequestMoveItemPosition` | `0xe91e00` | same | §2.2 |
| Dispose item | `Inventory::RequestDisposeItem` | `0xe91c50` | same | §2.2 |
| Equip | `BControlledPlayingActions::RequestEquipPacket` | `0x10d4070` | same | §2.3 |
| Unequip | `BControlledPlayingActions::RequestUnequipPacket` | `0x10d3ec0` | same | §2.3 |
| Sort / stack (move list) | `Inventory::requestRearrange` | `0xea9d10` | same | §2.4 |
| Use grocery | `Inventory::RequestUseGroceryItem` | `0xeda780` | same | §3.1 |
| Attach gem | `Inventory::RequestAttachGem` | `0xe91a30` | same | §3.3 |
| Enchant | `IntensionPanel::RequestPacket` | `0xe9c170` | same | §3.4 |
| Transform | `RequestTransformMultipleFeedItem` | `0xe8a490` | same | §3.2 |
| Buy at NPC | `ItemStorePanel::RequestBuyItem` | `0xd23960` | same | §4 |
| Sell at NPC | `ItemStorePanel::RequestSellItem` | `0xd23270` | same | §4 |
| Put into wardrobe | `ClosetGroup::MoveInventoryToClostReq` | `0x10dbb10` | same | §5 |
| Take from wardrobe | `ClosetGroup::MoveClosetToInventoryReq` | `0x10db6a0` | same | §5 |
| Goods warehouse take | `GoodsInventory::requestTakeWarehouseGoodsItemInQueue` | `0xf39550` | same | §6 |
| Expand inventory | `ExpandInvenSlotConfirmPanel::ProgressComplete` | `0xfdb7e0` | same | §1.2 |

Search anchors: string `"Msg.Item.AcquireItem.Acquire"`, `"Msg.Common.DoNotNow"`, `"Msg.Item.DisposeItemFail.NotEnoughItem"`, `"Msg.Item.MoveInventoryToWardrobe"`, `"Msg.PutItemIntoCloset.Fail.ByRule"`, log tag string `0x15A198C` ("Add"/"Remove"), table `Inventory::DEPOT_CAPACITY` `0x1955518`, T_SUB table `0x1496540`.

## 1. Inventory model

### 1.1 Depots (ItemDepot, 419 GM)

| Value | Name | Capacity (381 `0x1955518`) |
| --- | --- | --- |
| 0 | INVALID | 0 |
| 1 | EQUIP_SLOT | 38 |
| 2 | BAG | 96 |
| 3 | WAREHOUSE | 48 |
| 4 | SYSTEM | 30 (grows at run time, writes to `0x1955528`) |
| 5 | TRADE | 8 |
| 6 | WARDROBE | 80 |
| 7 | RANDOM_BOX | 16 |
| 8 | PREMIUM_DEPOSITORY | 80 |
| 9 | BADGE_DEPOSITORY | 48 |
| 10–12 | BADGE_EQUIP_SLOT_1..3 | 26 each |
| 13 | CARD_TRANSIT | (not in table) |
| 14 | COUNT | — |

The table gives the maximum. The real open size is in `CreatureProperty` field 145 `inventory_size` and 147 `wardrobe_size` (see 08). Money is field 109 `money`, 110 `money_diff`. Badge page is field 216. Equip appearance is fields 368–401.

### 1.2 Expand inventory

- C2S 1328 `ExpandInventoryReq`: +16 depot i8, +24 npcId i64, +32 consumeItems list. Sender `ExpandInvenSlotConfirmPanel::ProgressComplete` `0xfdb7e0`.
- S2C 1329 `ExpandInventoryOk`: inventoryType i8. Handler `0xef03d0`. The new size comes in the creature property sync (field 145/147) ❓.

### 1.3 Item instance (BItem and property tables)

`Property::FInfo` is 20 bytes (id, type, size, name, offset). The class vtable slot 1 gives the table id; slot 2 gives the field count+1.

| Table | ItemClass (419 GM) | GS_ItemProperty variant | Table id | FIELD_INFO | Fields | Class vtable | Sync vtable / FieldInfo | Sync size | BItem alloc in `0xed3820` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| WeaponProperty | 1 | 2 | 3 | `0x19f9168` | 48 | `0x165cfd8` | `0x179c31c` / `0x19f9668` | 0xE8 | 0x108 (ctor `0x75c250`) |
| CostumeProperty | 2 | 3 | 4 | `0x19f56a8` | 21 | `0x165cfe8` | `0x179c328` / `0x19f4aa0` | 0x50 | 0x60 ❓ |
| GroceryProperty | 3 | 4 | 9 | `0x19f9528` | 8 | `0x17c8808` | `0x179c334` / `0x19f95c8` | 0x20 | 0x34 ❓ |
| GemProperty | 4 | 5 | 11 | `0x19f5860` | 35 | `0x165cff8` | `0x179c364` / `0x19fa108` | 0xB0 | 0xC0 ❓ |
| AccessoryProperty | 5 | 6 | 15 | `0x19f5310` | 35 | `0x165d008` | `0x179c370` / `0x19f5d10` | 0xB8 | 0xC8 (ctor `0x1246ef0`) ❓ |

The ❓ allocation sizes are from `operator new` calls in the case blocks of `0xed3820` (case 3, case 5, case 6). I did not check each ctor.

Shared fields (all tables): `depot` INT8 off 4, `pos` INT16 off 6, `data_id` INT32 off 8, `level` INT8 off 12, `exp` INT32 off 16. In memory the sync object stores a field at `+8 + offset` (depot +12, pos +14, data_id +16, level +20).

- Grocery: depot, pos, data_id, level, exp, sequestration, count, enchant_point.
- Weapon adds: durability, appearance_item_*, spirit_*, gem_1..8, enchant_gem_1/2 (level, exp), enchant_point, enchant_consecutive_trial, enchant_crack, enchant_restore, random_effect_group_id_1..5, random_effect_id_1..5, transform_failure_exp (sync +66), spirit_data_id (sync +68).
- Costume adds: guild_id, custom1..8.
- All fields: `09-items-properties.tsv`.

Helpers: `GetPropertyFieldInfo<T>` `0x558280` (Weapon), `0x558310` (Grocery), `0x5583a0` (Costume), `0x558430` (Gem). `formatItemProperty<T>` `0x5563c0`, `0x556860`, `0x556d00`, `0x5571a0`. Copy ctors `0x534880`, `0x534ab0`, `0x534b60`, `0x534cd0`.

Other FInfo tables with no known user: `0x19fafd0`, `0x19fb070` (7 fields, depot..sequestration); `0x19f48c0`, `0x19fa478` (23 fields, item_id_1..new_stone_pos, slate ❓).

**GS_ItemProperty** (T_SUB, variant table `0x1496540`, 20-byte Def entries):

| Variant | Content | Create function |
| --- | --- | --- |
| 0 | null | — |
| 1 | empty PfStruct | `0x57b5d0` |
| 2 | Weapon PROP_SYNC | `0x57b450` |
| 3 | Costume PROP_SYNC | `0x57b490` |
| 4 | Grocery PROP_SYNC | `0x57b4d0` |
| 5 | Gem PROP_SYNC | `0x57b550` |
| 6 | Accessory PROP_SYNC | `0x57b590` |
| 7 | 6 fields: INT64, INT16, STRING, STRING, INT16, INT16 ❓ | `0x5a1df0` |

Variant = ItemClass + 1 ❓ (the order agrees). Used by cmd 81 `RebuyItemListOk`, 325 `AppendTradeSuggestionOther`, 375 `RetrievePostOk` and 1332.

### 1.4 Inventory object layout (381)

| Offset | Content | Proof |
| --- | --- | --- |
| BPlayer+0xB50 | `Inventory*` | BPlayer ctor `0xb8b4c9` |
| +0 | vtable `0x15a86e4` (172 functions) | ctor `0xe7d290` |
| +12 + 16·depot | `BItem**` first | `0xe616a0` |
| +16 + 16·depot | `BItem**` last | `0xe616a0` |
| +76/+80 | depot 4 (SYSTEM) vector | `0xe616a0` |
| +0x138 / +0x139 / +0x13A | move-list / equip / unequip busy flags | `0xed3820` |
| +0x1A0..+0x1B0 | last MoveItem src/dst | `0xe91e00`, cleared on 1340 |
| +0x1E4 | flag, cleared on 1332 | `0xed3820` |
| +0x1E5 | request lock byte | `0xe91e00` |
| +0x1E8 | pending cmd id (−1 = none) | `0xe91e00` |
| +0x1EC | cleared on 1141 / 1142 (badge page) | `0xed3820` |

Slots are 1-based: slot `pos` is element `pos−1`.

Set slot `0xe616a0` (item, depot, pos):
- Depot 4, pos 0: release the item.
- Depot 4: grow the vector (double size).
- Other depot: require `Inventory::IsValidDepot` `0xe30e60` and 1 ≤ pos ≤ `DEPOT_CAPACITY[depot]`. Delete the old item, store the new item, set item owner = inventory, call `Inventory::setLockExpirationItem` `0xe5d420`. Log "Add" or "Remove".

Other functions: `GetItem` `0xe3dd40`, `FindEmptySlot` `0xe3d230`, `GetSpareInvenSlotCnt` `0xe3cc80`, `IsValidPosition` `0xe30eb0`, `IsEquipable` `0xea5c30`, `GetEquipSlotItem` `0xe34770`, `IsTradableItem` `0xe4c5c0`, `CanInsertItems` `0xe495b0`, `checkPlayerStatus` `0xe34450`.

### 1.5 Request lock (state machine)

| State | +0x1E5 | +0x1E8 | Enter | Leave |
| --- | --- | --- | --- | --- |
| Idle | 0 | −1 | start, or reply | sender succeeds |
| Pending | 1 | cmd | `Request*` sends | `0xed3820` gets a cmd from the reset list |

While Pending, senders return 0 and send nothing (`0xe91e00`, `0xe91c50`, `0xd23960`). I found no timer that releases the lock ❓.

Reset list (in `0xed3820`): 323, 446, 452, 454, 464, 523, 526, 527, 534, 831, 832, 941, 942, 948, 949, 953, 1011, 1012, 1227, 1235, 1241, 1259, 1267, 1269, 1293, 1295, 1307–1309, 1312, 1313, 1317–1319, 1332, 1333, 1335, 1337, 1339, 1340, 1342, 1344, 1347, 1349, 1351, 1353, 1355, 1357, 1358, 1360, 1362, 1363, 1365, 1366, 1368, 1370, 1371, 1373, 1374, 1376, 1377, 1379, 1380, 1382, 1384, 1386, 1388, 1393, 1394, 1397, 1401, 1403, 1404, 1406, 1407, 1409, 1410, 1413, 1415, 1417, 1419, 1582, 1583. Column `inventory_lock_reset` in `09-items-commands.tsv`.

## 2. Inventory packets

### 2.1 S2C 1332 InventoryItemListOk

- Handler `0xef1430` → `BPlayer::ProcessNetObjectEvent` `0xbbfe10` (switch on cmd−8, byte table `byte_BC183C`) → Inventory vtable slot 3 `0xed3820`.
- Fields: +16 `itemReason` i8 (ItemReason, 419 GM); +20 `item` REPEAT (def `0x1495668` GR_InventoryItemListRptBody). Each element has `item` T_SUB at +8 → GS_ItemProperty.

Client behaviour (`0xed3820`):
1. Reset the lock.
2. For each element, switch on the variant (element+13). Read depot, pos and data_id from the sync object.
3. data_id == 0: clear the slot (`0xe616a0` with no item).
4. Else: allocate a BItem (table §1.3), `BItem::Init(sync)`, call `0xe616a0`.
5. Extra work for reasons 1, 3, 4, 5, 6, 7, 26, 57, 58, 60, 62, 63, 68.
6. Acquire reasons 1, 26, 57, 58, 60, 68: `BPlayer::OnNotifyItemAcquired`, message `"Msg.Item.AcquireItem.Acquire"` / `".Fx"` (`World::InvokeGameMessage`), `PlayReport::AddAcquiredItem`.

BPlayer also gives 1332 to the open loot pouch (`World::GetItemPouch` `0x732140`), to `SkillSkin::ProcessNetObjectEvent` and to `BCollectingManager`. Panels update in `InventoryEquipPanel::ProcessNetObjectEvent_InventoryItemListOkCmd` `0xdfb3a0` and `NewInventoryPanel2::ProcessNetObjectEvent` `0xe2faa0`.

Stack change = same depot/pos with a new `count`. Move = one element clears src (data_id 0), one sets dst. Equip = item in depot 1 ❓ (I inferred these from the set-slot logic, not from a capture).

C2S 1331 `InventoryItemListReq` (sender `0xe8d480`) asks for the full list.

ItemReason (419 GM, 90 values, all in TSV): 0 REQUEST, 1 ACQUIRE, 2 DISPOSE, 3 MOVE, 4 EQUIP, 5 UNEQUIP, 6 ATTACH_GEM_TO_WEAPON, 16 ENCHANT_TO_ITEM, 18 CONSUME, 22 SELL, 24 SEAL, 25 DECOMPOSE, 31 TRADE_WITH_NPC, 36 TRANSFORM, 37 GROWTH, 39 AWAKENING, 41 EXPAND_INVENTORY, 48 CLOSET, 52 EXCHANGE, 70 SET_ITEM_USED, 79 POLISH_TO_ITEM, 80 ITEM_FUSION, 89 IMPROVE.

### 2.2 Move and dispose

| Cmd | Name | Dir | Fields (offset name type) | 381 code |
| --- | --- | --- | --- | --- |
| 1334 | DisposeItemReq | C2S | 16 bagPos i16, 18 count i16 | `0xe91c50` |
| 1335 | DisposeItemFail ❓ | S2C | — | `0xef1060` |
| 1336 | MoveItemReq | C2S | 16 srcDepot i8, 18 srcPos i16, 20 count i16, 22 destDepot i8, 24 destPos i16, 32 npcId i64 | `0xe91e00` |
| 1337 | MoveItemFail | S2C | — | `0xef1010` |
| 1338 | MoveItemListReq | C2S | 16 npcId i64, 24 srcDepot i8, 25 destDepot i8, 28 stackList, 40 moveList (both GR_MoveItemPosCountRptBody `0x14959ec`: srcPos i16, srcCount i16, destPos i16) | `0xea9d10`, `0xeba8d0` |
| 1339 | MoveItemList reply | S2C | — | `0xef0d90` |

`RequestMoveItemPosition` `0xe91e00`: lock set → 0; `checkPlayerStatus` fails → `"Msg.Common.DoNotNow"`; alloc 40 bytes, cmd 1336, `BNSClient::SendToGamed`; then +0x1E8 = 1336, +0x1E5 = 1, save src/dst. `RequestDisposeItem` `0xe91c50` uses the same lock; error `"Msg.Item.DisposeItemFail.NotEnoughItem"`.

### 2.3 Equip and unequip

| Cmd | Name | Dir | Fields | 381 code |
| --- | --- | --- | --- | --- |
| 1354 | EquipItemReq | C2S | 16 depot i8, 18 pos i16, 20 equipDepot i8, 22 equipPos i16 | `0x10d4070` |
| 1355 | EquipItemFail ❓ | S2C | — | `0xef0fe0` |
| 1356 | EquipItemListReq | C2S | list {depot, pos, equipPos} | — |
| 1357 | EquipItemList reply | S2C | — | `0xeeba20` |
| 1359 | UnequipItemReq | C2S | 16 equipDepot, 18 equipPos, 20 targetDepot, 22 targetPos | `0x10d3ec0` |
| 1360 / 1362 | Unequip replies | S2C | — | `0xef0e60`, `0xeeba60` |

`RequestEquipPacket` `0x10d4070` makes 8 checks before it sends ❓ (not listed). High-level: `Inventory::RequestEquip` `0xea6fc0`, `0xea87f0`; `RequestUnequip` `0xe9b050`, `0xe9c140`. Fail reasons: `Inventory::EquipFailReason` (26 values, 419 GM, in TSV): 0 NONE, 1 KISMET_PLAY, 2 PLAYER_DEAD, …, 24 INTERNAL.

### 2.4 Sort and stack

`Inventory::requestRearrange` `0xea9d10` uses `sortSortableItemList` `0xe48290` and `mergeStackableItemBySortableItemList` `0xe3c3c0`, then sends 1338 with a stackList (merges) and a moveList.

### 2.5 Field items and pouches

World drop and pickup items travel in `GS_Actions` (variant table `0x1496720`, 103 variants) in cmd 6/7 Broadcast(Private)Action; see 08. Senders: `RequestOpenPouch` `0x10d5290` (1296), `RequestOpenPrivate` `0x10d5120` (1299), `RequestClosePouch` `0x10d51e0` (1303), `RequestPickupItem` `0x10d4b50` / `0x10d4fd0` (1306), `RequestDistributeItem` `0x10d4560` (1316), `0x10d36c0` (1242, 1243, 1245), `RequestPickup` `0x10d4450` (1247), `0x10f7ae0` (1236/1237). Handlers 1297 `0xef09a0`, 1300 `0xef0930`, 1308 `0xef07b0`, 1310 `0xef0730`. `ItemPouch::ProcessNetObjectEvent` `0xd30de0`.

## 3. Use and upgrade

### 3.1 Use

`Inventory::RequestUseGroceryItem` `0xeda780`, `RequestUseGroceryItemToGround` `0xea6b70`, `CanUseGrocery` `0xe8c7a0`. Fail reasons `Inventory::UseGroceryFailReason` (47) and `InventoryItemUnUseableReason` (27), 419 GM. `RequestRepairAll` `0xe908d0`, `GetEstimatedMoneyToRepairAll` `0xe4cda0`. Set-used 1426 `SetItemUsedReq` {depot, pos} from `BCollectingManager::SetUsedItem` `0x10fc720`.

### 3.2 Upgrade senders

| Cmd | Purpose | 381 sender |
| --- | --- | --- |
| 830 | apply spirit | `Inventory::RequestApplySpiritItem` `0xe8f120` |
| 947 | appearance change | `Inventory::RequestAppearanceChange` `0xe89240` |
| 1372 | (NewInventoryPanel2) | `0xe2c860` |
| 1375, 1378, 1381, 1383, 1385, 1387 | item operations | `0xe1c6d0`, `0xe8fc30`, `0xe8f940`, `0xe8fdf0`, `0xe90740`, `0x10e9b60` |
| 1392 | seal | `Inventory::RequestSeal` `0xe90170` |
| 1395 | decompose | `RequestDecomposeByItem2` `0xe91630`, `0xe91280` |
| 1398 | (GoodsInventory) | `0xf3a690` |
| 1402 | growth | `0xe8e3f0` |
| 1405 | awakening | `0xe8d680` |
| 1408 | — | `0xe88760` |
| 1414 / 1416 / 1418 | lock / unlock / — | `0xe8f030`, `0xe8ef50`, `0xe8ee60` |
| 1445 | fusion | `0xdb43c0` (false name `Teleport::FindTeleportByZone`) |
| 1581 | transform | `0xe87ad0`, `0xe87fa0`, `0xe8a490`, `0xe8b540` |
| 1008 / 1010 | sell old item / exchange | `ItemCleanerPanel` `0xdb5270`, `0xdb4a50` |

Replies in the lock reset list (§1.5). Handlers: 1382 `0xef0b30`, 1393 `0xef0a70`, 1403 `0xef0300`, 1406 `0xef0290`, 1582 `0xef20f0`. Panels: `ItemTransformPanel` `0xd5c8f0`, `ItemGrowth2Panel` `0xe0a890`, `ItemFusionPanel` `0xe0d470`.

### 3.3 Gems

`RequestAttachGem` `0xe91a30` (1364), `RequestDetachGem` `0xe918e0` (1367), `0xe90fc0` (1369), `RequestAttachEquipGemPiece` `0xe8f570` (1400). Handlers 1365 `0xef0e20`, 1368 `0xef0d00`. Gem slots: `WeaponProperty::Gem_Pos` (8 values, 419 GM). `WeaponGemPanel` `0x7adc20`.

### 3.4 Enchant (intension)

`IntensionPanel::RequestPacket` `0xe9c170` sends 1430, 1433, 1436, 1439. `IntensionPanel::requestGrindOption` `0xe92230` sends 1442 (polish). `IntensionManager` `0xeacad0`. Handlers 1431 `0xee6a80`, 1446 `0xee6b80`.

## 4. NPC shop

| Cmd | Name | Dir | Fields | 381 code |
| --- | --- | --- | --- | --- |
| 80 / 81 | RebuyItemList / Ok | C2S / S2C | 81 holds GS_ItemProperty | `ItemStorePanel::Show` `0xd6cf70` / `0xef0f20` |
| 1341 | BuyReq | C2S | seller i64, items {index i8, count i16} | no sender found |
| 1342 | BuyFail | S2C | — | `0xef0fb0` |
| 1343 | Buy2Req | C2S | 16 seller i64, 24 storeIndex i8, 25 itemIndex i8, 26 count i16, 28 consumeItems {pos i16, count i16} (`0x1495a3c`) | `0xd23960` |
| 1345 | MysteriousBuy | C2S | — | `0xd23960` |
| 1348 | SellReq | C2S | 16 buyer i64, 24 items {pos, count} | `0xd23270`, `0xd22f00` |
| 1349 | SellFail | S2C | — | `0xef0f50` |
| 1350 / 1351 | Rebuy | C2S / S2C | ❓ | not checked |
| 1352 / 1353 | TradeWithNpc | C2S / S2C | ❓ | not checked |

`RequestBuyItem` `0xd23960`: check lock +0x1E5, `isNeedItemBrandForBuy`, `checkExchangeWeapon`, `FindItemLimitedCount`. storeIndex = `getSelectedUnLocatedStoreKey`. For a mysterious store it sends 1345 (`MysteriousStoreManager` at World+3068, `0xc60ce0`). The bought item and money change come in 1332 and the creature property sync. Random store: `RandomStoreManager` `0x9d7180`, `0x9d6ec0`, `0x9d6c40` (1569, 1572, 1575). `ItemStorePanel::ProcessNetObjectEvent` `0xd6cc00`.

## 5. Wardrobe (closet) and costumes

| Cmd | Name | Dir | Fields | 381 code |
| --- | --- | --- | --- | --- |
| 934 | PutItemIntoClosetReq | C2S | bagPos i16 | `ClosetGroup::MoveInventoryToClostReq` `0x10dbb10` |
| 935 | PutItemIntoClosetOk | S2C | closetGroupID i32 | `0xef1b30` → `ClosetGroup::add` |
| 936 | PutItemIntoClosetFail | S2C | error code | `0x1113050` |
| 937 | TakeItemFromClosetReq | C2S | closetGroupID i32, targetBagPos i16 | `0x10db6a0` |
| 938 | TakeItemFromClosetOk | S2C | closetGroupID i32 | `0xef1a30` → `ClosetGroup::remove` |
| 939 | (stop ping-pong) | S2C | — | `UiStateGame::StopPingPongWardrobe` |
| 1118 / 1119 | SetClosetItemUsed | C2S / S2C | closetGroupIdList {id i32} | `0x10fc720` / `0xee7ec0` |
| 1428 / 1429 | CompleteClosetCollecting | C2S / S2C | dataId, items {depotType, depot, pos, isAppearance, closetGroupId} | not checked |
| 1584 / 1585 | BuyCollectionItems | C2S / S2C | totalGoodsPrice VBIN, shoppingList {goodsNumber, goodsPrice VBIN, closetGroupKey i64, collectionItemKey i64} / purchasedList | `ClosetGroup::BuyClosetItemReq` `0x10db030` |

`ClosetGroup::ProcessNetObjectEvent` `0x1113050`:
- 936 codes: 2169 `"Msg.PutItemIntoCloset.Fail.ByRule"`, 2170 `.InvalidItem`, 2171 `.FullCloset`, 2172 `.DuplicatedClosetGroup`.
- 1585: `"Msg.Collection.Purchase.%d"`, `"Msg.SecondaryMoney.Loss"`.
- Busy messages: `"Msg.Item.MoveInventoryToWardrobe.Fail.Working(2)"`, `"Msg.Item.MoveWardrobeToInventory.Fail.Working(2)"`.

Depot 6 WARDROBE (80 slots) also exists in the depot table. `WardrobePanel` vtable `0x1696fd8`, events `0x7b8110`. Costume data is `CostumeProperty` (§1.3).

## 6. Warehouse

- **NPC warehouse** = depot 3 (48 slots). Move with 1336 or 1338 and set `npcId`. `WarehousePanel` vtable `0x1696748`, events `0x791320`.
- **Account goods warehouse** (`GoodsInventory`, events `0xf4a460`, route `BAccount::ProcessNetObjectEvent` `0x128c650`):

| Cmd | Name | Fields | 381 code |
| --- | --- | --- | --- |
| 805 / 806 | RetrieveWarehouseItemList Req / Ok | search_type, search_key, language_code / goods list | `0xf39aa0`, `0xf3a010` / `0xef2ab0` |
| 808 | AccountInventoryChangeNotify | notification_type, item_list {item_instance_id, label_id, item_data_id}, item_list_remain, goods_label_id_min | `0xef27b0` |
| 1411 / 1412 | TakeWarehouseItem Req / Ok | label_id, item_instance_id, item_data_id / label_id, item_instance_id | `0xf39550` / `0xef25d0` |
| 802 / 803 | LobbyService item list | — | `0xf39740` |
| 793, 796, 799 | MTX goods count / list / take | — | `0xf39d50`, `0xf39c00`, `0xf39990` |

## 7. Worked example: move a bag item to the NPC warehouse

1. The player drags bag slot 5 (count 3) to warehouse slot 1 at NPC `0x…1234`.
2. `RequestMoveItemPosition` `0xe91e00`: lock is 0; status is good. It sends 1336: srcDepot 2, srcPos 5, count 3, destDepot 3, destPos 1, npcId. It sets +0x1E8 = 1336, +0x1E5 = 1.
3. The server replies 1332, itemReason 3 (MOVE), two elements: (depot 2, pos 5, data_id 0) and (depot 3, pos 1, data_id N, count 3).
4. `0xed3820` resets the lock, clears bag slot 5 and creates a new BItem in warehouse slot 1 with `0xe616a0`.
5. On error the server sends 1337 `MoveItemFail`; `0xed3820` resets the lock too.

## 8. Open items (❓)

- BItem allocation sizes for variants 3–6 and the exact ctors in `0xed3820`.
- GS_ItemProperty variant 7 content and user (`0x5a1df0`).
- The 8 checks in `RequestEquipPacket` `0x10d4070`.
- Field layout of the Fail packets (error code offset).
- Rebuy 1350/1351, TradeWithNpc 1352/1353, CompleteClosetCollecting 1428/1429 flows.
- No timer for the request lock was found.
- Senders not found: 833, 1325, 1341, 1361.
