//! Doc 12b part 1: manufacture stub (1325), equipment (1354–1361), craft
//! (1512–1527) and item lock/used (1414–1426) entry functions. Build 381.8867.
//!
//! Entry functions decode, gate and queue an `ItemTask`. Task runners, item
//! virtual getters/setters and inventory batches are adapter boundaries.
//! Abort craft (1521) runs inline in the native entry and is ported in full.
use super::*;

pub const ITEM2_COMMANDS: &[u16] = &[
    1325, 1354, 1356, 1359, 1361, 1414, 1416, 1418, 1426, 1512, 1515, 1518, 1521, 1524, 1527,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemTaskKind {
    /// Kind 7. Operation +256: 1 for destination 10..12, else 0.
    Equip {
        src_selector: u8,
        src_slot: u16,
        dst_selector: u8,
        dst_slot: u16,
        operation: u32,
    },
    /// Kind 7, operation 2. Entries: selector, source slot, destination slot.
    EquipList { entries: Vec<(u8, u16, u16)> },
    /// Kind 8. Operation +228: 1 for source 10..12, else 0.
    Unequip {
        src_selector: u8,
        src_slot: u16,
        dst_selector: u8,
        dst_slot: u16,
        operation: u32,
    },
    /// Kind 8, operation 2. Entries: equipped slot, destination operand.
    UnequipList { entries: Vec<(u16, u16)> },
    /// Kind 274. Count +170 is signed; only positive counts read entries.
    RegisterCraft {
        recipe: i32,
        count: i16,
        entries: Vec<(u16, u16)>,
    },
    /// Kind 275.
    TakeCraft { craft_id: u64 },
    /// Kind 276.
    JoinCraftUnion { selector: i8, target: i64 },
    /// Kind 277.
    LeaveCraftUnion { selector: i8 },
    /// Kind 177.
    Lock { container: u8, slot: u16 },
    /// Kind 178.
    Unlock { container: u8, slot: u16 },
    /// Kind 179. No client expiry field.
    SetLockExpiration { container: u8, slot: u16 },
    /// Kind 315. Entries in request order (1..16, duplicates allowed).
    SetUsed { entries: Vec<(u8, u16)> },
    /// Doc 12b part 2 (closet, inventory, repair, decompose, gem, seal).
    Inventory(InventoryTask),
    /// Doc 12c part 1 (badge, slate).
    BadgeSlate(BadgeSlateTask),
    /// Doc 12c part 2 (item use, field items, pouches, crafting entries).
    Use(ItemUseTask),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemTask {
    pub command: u16,
    pub caster: u64,
    pub zone: u64,
    pub kind: ItemTaskKind,
}

/// One PC craft record (vector at PC +39760, lock at +39752).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftRecord {
    /// Record +24.
    pub id: u64,
    /// Record byte +144.
    pub state: u8,
    /// Record qword[56] (inclusive start).
    pub start: i64,
    /// Record qword[64] (exclusive end).
    pub end: i64,
}

pub(super) fn dispatch_item2(ctx: &mut Context<'_>, command: u16, req: &[u8]) -> Outcome {
    match command {
        1325 => handle_manufacture_item(ctx, req),
        1354 => handle_equip_item(ctx, req),
        1356 => handle_equip_item_list(ctx, req),
        1359 => handle_unequip_item(ctx, req),
        1361 => handle_unequip_item_list(ctx, req),
        1512 => handle_retrieve_craft_list(ctx, req),
        1515 => handle_register_craft(ctx, req),
        1518 => handle_take_craft(ctx, req),
        1521 => handle_abort_craft(ctx, req),
        1524 => handle_join_craft_union(ctx, req),
        1527 => handle_leave_craft_union(ctx, req),
        1414 => handle_lock_item(ctx, req),
        1416 => handle_unlock_item(ctx, req),
        1418 => handle_set_item_lock_expiration(ctx, req),
        1426 => handle_set_item_used(ctx, req),
        _ => unreachable!("membership checked"),
    }
}

/// parity: 1325 12b-item-handlers.md#command-1325--g_manufactureitemreqcmd ported
/// `0x14001AFE0` returns true. Doc 12b: do not implement manufacture.
pub fn handle_manufacture_item(_: &mut Context<'_>, _: &[u8]) -> Outcome {
    Outcome::default()
}

const UNCHECKED_DECODE: &str = "❓ unchecked item decode (short body)";

/// Common entry tail: zone (fail), allocate, PC kind 1 (log only), queue.
/// `fail` is the reply for zone/queue failure. `kind_fails` selects whether a
/// wrong PC kind also sends `fail` (initializer failure) or only logs.
fn item_queue(
    ctx: &mut Context<'_>,
    command: u16,
    fail: Outcome,
    kind_fails: bool,
    kind: ItemTaskKind,
) -> Outcome {
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return fail;
    };
    if pc.native_kind != 1 {
        if kind_fails {
            let mut out = fail;
            out.logs.push("item task: caster is not a PC");
            return out;
        }
        return Outcome::log("item task: caster is not a PC");
    }
    let task = ItemTask {
        command,
        caster: pc.object_id,
        zone,
        kind,
    };
    match ctx.services.queue_item(&task) {
        Gate::Pass => Outcome {
            changes: vec![
                StateChange::Queued(command),
                StateChange::ItemTaskQueued(task),
            ],
            ..Outcome::default()
        },
        Gate::Fail => fail,
        Gate::Unknown => Outcome::unsupported("❓ item task allocation/queue adapter"),
    }
}

/// `hQhQh`: selector byte, slot, selector byte, slot.
fn single_move(req: &[u8]) -> Option<(u8, u16, u8, u16)> {
    let mut r = Reader(req);
    Some((r.byte()?, r.short()?, r.byte()?, r.short()?))
}

/// parity: 1354 12b-item-handlers.md#command-1354--g_equipitemreqcmd partial
pub fn handle_equip_item(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some((src_selector, src_slot, dst_selector, dst_slot)) = single_move(req) else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    if !matches!(dst_selector, 1 | 10..=12) || !matches!(src_selector, 1 | 2 | 9..=12) {
        return Outcome::reason(1355, 1);
    }
    let operation = u32::from((10..=12).contains(&dst_selector));
    let kind = ItemTaskKind::Equip {
        src_selector,
        src_slot,
        dst_selector,
        dst_slot,
        operation,
    };
    item_queue(ctx, 1354, Outcome::reason(1355, 1), false, kind)
}
/// parity: 1359 12b-item-handlers.md#command-1359--g_unequipitemreqcmd partial
pub fn handle_unequip_item(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some((src_selector, src_slot, dst_selector, dst_slot)) = single_move(req) else {
        return Outcome::unsupported(UNCHECKED_DECODE);
    };
    if !matches!(src_selector, 1 | 10..=12) || !matches!(dst_selector, 2 | 9) {
        return Outcome::reason(1360, 1);
    }
    let operation = u32::from((10..=12).contains(&src_selector));
    let kind = ItemTaskKind::Unequip {
        src_selector,
        src_slot,
        dst_selector,
        dst_slot,
        operation,
    };
    item_queue(ctx, 1359, Outcome::reason(1360, 1), false, kind)
}
/// Counted list: signed-short count, unsigned count ≥ 16 rejects.
fn counted<T>(req: &[u8], mut entry: impl FnMut(&mut Reader<'_>) -> Option<T>) -> Option<Vec<T>> {
    let mut r = Reader(req);
    let count = r.short()?;
    if count >= 16 {
        return None;
    }
    (0..count).map(|_| entry(&mut r)).collect()
}
/// parity: 1356 12b-item-handlers.md#command-1356--g_equipitemlistreqcmd partial
pub fn handle_equip_item_list(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some(entries) = counted(req, |r| Some((r.byte()?, r.short()?, r.short()?))) else {
        return Outcome::reason(1355, 2);
    };
    let kind = ItemTaskKind::EquipList { entries };
    item_queue(ctx, 1356, Outcome::reason(1355, 1), false, kind)
}
/// parity: 1361 12b-item-handlers.md#command-1361--g_unequipitemlistreqcmd partial
pub fn handle_unequip_item_list(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let Some(entries) = counted(req, |r| Some((r.short()?, r.short()?))) else {
        return Outcome::reason(1360, 2);
    };
    let kind = ItemTaskKind::UnequipList { entries };
    item_queue(ctx, 1361, Outcome::reason(1360, 1), false, kind)
}

/// parity: 1512 12b-item-handlers.md#command-1512--g_retrievecraftlistreqcmd partial
pub fn handle_retrieve_craft_list(_: &mut Context<'_>, _: &[u8]) -> Outcome {
    Outcome::unsupported("❓ craft list builder 0x14060EC20 record fields")
}

/// `hEi` / `hEq` failure bodies: reason then value.
fn reason_i32(command: u16, reason: u16, value: i32) -> Outcome {
    let mut body = reason.to_le_bytes().to_vec();
    body.extend(value.to_le_bytes());
    Outcome::reply(command, body)
}
pub(super) fn reason_u64(command: u16, reason: u16, value: u64) -> Outcome {
    let mut body = reason.to_le_bytes().to_vec();
    body.extend(value.to_le_bytes());
    Outcome::reply(command, body)
}
fn reason_byte(command: u16, reason: u16, value: i8) -> Outcome {
    let mut body = reason.to_le_bytes().to_vec();
    body.push(value as u8);
    Outcome::reply(command, body)
}
/// Craft tasks dispatch through `0x140603BD0`. Its result is not tested at
/// entry, so anything but Pass is a boundary.
pub(super) fn craft_dispatch(ctx: &mut Context<'_>, task: ItemTask) -> Outcome {
    match ctx.services.queue_item(&task) {
        Gate::Pass => Outcome {
            changes: vec![
                StateChange::Queued(task.command),
                StateChange::ItemTaskQueued(task),
            ],
            ..Outcome::default()
        },
        _ => Outcome::unsupported("❓ craft task dispatch 0x140603BD0"),
    }
}

/// parity: 1515 12b-item-handlers.md#command-1515--g_registercraftreqcmd partial
pub fn handle_register_craft(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // Every entry failure: 1517 `hEi`, reason 2, recipe 0.
    let fail = || reason_i32(1517, 2, 0);
    let pc = ctx.pc().clone();
    // PC +20696 user with +20752 = 0, then user DWORD +120 = 0.
    if pc.t2_blocked || !pc.user_present || pc.party_user_blocked {
        return fail();
    }
    let Some(zone) = pc.zone else {
        return fail();
    };
    if pc.native_kind != 1 {
        let mut out = fail();
        out.logs.push("register craft: caster is not a PC");
        return out;
    }
    let mut r = Reader(req);
    let Some(recipe) = r.int().map(|v| v as i32).filter(|v| *v != 0) else {
        return fail();
    };
    let Some(count) = r.short().map(|c| c as i16).filter(|c| *c <= 9) else {
        return fail();
    };
    let mut entries = vec![];
    for _ in 0..count.max(0) {
        let (Some(a), Some(b)) = (r.short(), r.short()) else {
            return fail();
        };
        entries.push((a, b));
    }
    if recipe <= 0 {
        return fail();
    }
    match ctx.services.craft_recipe_exists(recipe) {
        Some(true) => {}
        Some(false) => return fail(),
        None => return Outcome::unsupported("❓ recipe table tm[603] lookup"),
    }
    let task = ItemTask {
        command: 1515,
        caster: pc.object_id,
        zone,
        kind: ItemTaskKind::RegisterCraft {
            recipe,
            count,
            entries,
        },
    };
    craft_dispatch(ctx, task)
}

/// parity: 1518 12b-item-handlers.md#command-1518--g_takecraftreqcmd partial
pub fn handle_take_craft(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    // The ID is initialized to zero before the unchecked decode.
    let craft_id = Reader(req).quad().unwrap_or(0);
    let fail = || reason_u64(1520, 2, craft_id);
    let pc = ctx.pc().clone();
    let Some(zone) = pc.zone else {
        return fail();
    };
    if craft_id == 0 {
        return fail();
    }
    if pc.native_kind != 1 {
        let mut out = fail();
        out.logs.push("take craft: caster is not a PC");
        return out;
    }
    let task = ItemTask {
        command: 1518,
        caster: pc.object_id,
        zone,
        kind: ItemTaskKind::TakeCraft { craft_id },
    };
    craft_dispatch(ctx, task)
}

/// parity: 1521 12b-item-handlers.md#command-1521--g_abortcraftreqcmd ported
pub fn handle_abort_craft(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let craft_id = Reader(req).quad().unwrap_or(0);
    let fail = reason_u64(1523, 3, craft_id);
    if craft_id == 0 {
        return fail;
    }
    let now = ctx.world.utc_seconds;
    let Some(records) = ctx.services.craft_records() else {
        return Outcome::unsupported("❓ PC craft record vector adapter");
    };
    let mut passed = false;
    for rec in records.iter_mut().flatten() {
        if rec.id != craft_id {
            continue;
        }
        if rec.state != 3 {
            break;
        }
        if rec.start <= now && now < rec.end {
            // State 6 is set before the backend request and never rolled back.
            rec.state = 6;
            passed = true;
            break;
        }
    }
    if !passed {
        return fail;
    }
    let pc = ctx.pc();
    Outcome {
        daemon_requests: vec![DaemonRequest::CraftAbort {
            routing: pc.routing,
            short_96: i32::from(pc.short_96),
            restriction: ctx.world.arena_value,
            object_id: pc.object_id,
            db_id: pc.db_id,
            craft_id,
        }],
        changes: vec![StateChange::CraftAborting(craft_id)],
        ..Outcome::default()
    }
}

/// parity: 1524 12b-item-handlers.md#command-1524--g_joincraftunionreqcmd partial
pub fn handle_join_craft_union(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if ctx.pc().t2_blocked {
        return reason_byte(1526, 7, 0);
    }
    let mut r = Reader(req);
    let selector = r.byte().unwrap_or(0) as i8;
    let target = r.quad().unwrap_or(0) as i64;
    let pc = ctx.pc().clone();
    let fail = reason_byte(1526, 2, selector);
    let Some(zone) = pc.zone else {
        let mut out = fail;
        out.logs.push("join craft union: PC zone is null");
        return out;
    };
    if selector <= 0 || target <= 0 || (selector as u8).wrapping_sub(7) <= 7 {
        return fail;
    }
    if pc.native_kind != 1 {
        let mut out = fail;
        out.logs.push("join craft union: caster is not a PC");
        return out;
    }
    let task = ItemTask {
        command: 1524,
        caster: pc.object_id,
        zone,
        kind: ItemTaskKind::JoinCraftUnion { selector, target },
    };
    craft_dispatch(ctx, task)
}
/// parity: 1527 12b-item-handlers.md#command-1527--g_leavecraftunionreqcmd partial
pub fn handle_leave_craft_union(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if ctx.pc().t2_blocked {
        return reason_byte(1529, 7, 0);
    }
    let selector = Reader(req).byte().unwrap_or(0) as i8;
    let pc = ctx.pc().clone();
    let fail = reason_byte(1529, 2, selector);
    let Some(zone) = pc.zone else {
        let mut out = fail;
        out.logs.push("leave craft union: PC zone is null");
        return out;
    };
    if selector <= 0 {
        return fail;
    }
    if pc.native_kind != 1 {
        let mut out = fail;
        out.logs.push("leave craft union: caster is not a PC");
        return out;
    }
    let task = ItemTask {
        command: 1527,
        caster: pc.object_id,
        zone,
        kind: ItemTaskKind::LeaveCraftUnion { selector },
    };
    craft_dispatch(ctx, task)
}

/// `hQh`: container byte, slot.
fn container_slot(req: &[u8]) -> Option<(u8, u16)> {
    let mut r = Reader(req);
    Some((r.byte()?, r.short()?))
}
/// parity: 1414 12b-item-handlers.md#command-1414--g_lockitemreqcmd partial
pub fn handle_lock_item(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if !ctx.world.item_lock_enabled {
        return Outcome::reason(1415, 2);
    }
    let Some((container, slot)) = container_slot(req) else {
        return Outcome::reason(1415, 2);
    };
    let kind = ItemTaskKind::Lock { container, slot };
    item_queue(ctx, 1414, Outcome::reason(1415, 1), true, kind)
}
/// parity: 1416 12b-item-handlers.md#command-1416--g_unlockitemreqcmd partial
pub fn handle_unlock_item(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let fail = || reason_i32(1417, 2, 0);
    if !ctx.world.item_lock_enabled {
        return fail();
    }
    let Some((container, slot)) = container_slot(req) else {
        return fail();
    };
    let kind = ItemTaskKind::Unlock { container, slot };
    item_queue(ctx, 1416, fail(), true, kind)
}
/// parity: 1418 12b-item-handlers.md#command-1418--g_setitemlockexpirationtimereqcmd partial
pub fn handle_set_item_lock_expiration(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if !ctx.world.item_lock_enabled {
        return Outcome::reason(1419, 2);
    }
    let Some((container, slot)) = container_slot(req) else {
        return Outcome::reason(1419, 2);
    };
    let kind = ItemTaskKind::SetLockExpiration { container, slot };
    item_queue(ctx, 1418, Outcome::reason(1419, 1), true, kind)
}
/// parity: 1426 12b-item-handlers.md#command-1426--g_setitemusedreqcmd partial
pub fn handle_set_item_used(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let mut r = Reader(req);
    let entries = r
        .short()
        .filter(|c| (1..=16).contains(c))
        .and_then(|c| (0..c).map(|_| Some((r.byte()?, r.short()?))).collect());
    let Some(entries) = entries else {
        return Outcome::reason(1427, 2);
    };
    let kind = ItemTaskKind::SetUsed { entries };
    item_queue(ctx, 1426, Outcome::reason(1427, 2), true, kind)
}
