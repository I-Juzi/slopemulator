use super::*;
use crate::validation::movement as validation;

/// parity: 1221 11-movement-handlers.md#1221--g_warptoposreqcmd--done ported
pub fn handle_warp_to_pos(_: &mut Context<'_>, _: &[u8]) -> Outcome {
    Outcome::default()
}

/// parity: 64 11-movement-handlers.md#64--g_movereqcmd--partial partial
pub fn handle_move(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let mut r = Reader(req);
    let decoded = (|| {
        let selector = r.byte()?;
        let start = [r.short()?, r.short()?, r.short()?];
        let end = [r.short()?, r.short()?, r.short()?];
        let uu = [r.int()?, r.int()?, r.int()?, r.int()?, r.int()?, r.int()?];
        Some((
            selector,
            start,
            end,
            uu,
            r.short()?,
            r.short()?,
            r.short()?,
            r.byte()?,
        ))
    })();
    let Some((selector, _start, _end, _uu, duration, _yaw, _key, _broadcast)) = decoded else {
        return correction(ctx, 2, 0, false);
    };
    // STOP inlined restricted movement count, call 0x1403895C7 -> 0x1403EAB50.
    let summon = match validation::selects_summon(
        selector,
        ctx.pc().summon.as_ref().map(|s| s.control_state),
    ) {
        Ok(value) => value,
        Err(_) => return correction(ctx, 2, selector, false),
    };
    let (kind, zone) = if summon {
        let s = ctx.pc().summon.as_ref().expect("validated summon");
        if s.has_entry && validation::selected_creature(s.native_kind, s.race).is_err() {
            return correction(ctx, 166, selector, true);
        }
        (s.native_kind, s.zone)
    } else {
        (ctx.pc().native_kind, ctx.pc().zone)
    };
    let Some(count) = r.short() else {
        return correction(ctx, 103, selector, summon);
    };
    if count > 12 {
        return correction(ctx, 103, selector, summon);
    }
    let mut types = Vec::new();
    for _ in 0..count {
        let row = (|| {
            Some((
                r.byte()?,
                r.byte()?,
                r.byte()?,
                r.byte()?,
                r.byte()?,
                r.byte()?,
                r.byte()?,
                r.short()?,
            ))
        })();
        let Some((code, sprint, start_time, start_cell, _dx, _dy, _dz, _jump)) = row else {
            return correction(ctx, 2, selector, summon);
        };
        types.push(validation::MoveType {
            code: code as i8,
            flag: sprint != 0,
            start_time: start_time as i8,
            start_cell: start_cell as i8,
        });
    }
    let Some(declared) = r.short() else {
        return correction(ctx, 2, selector, summon);
    };
    if declared > 36 {
        return correction(ctx, 2, selector, summon);
    }
    let mut cells = Vec::new();
    for _ in 0..declared {
        let (Some(x), Some(y), Some(z)) = (r.short(), r.short(), r.short()) else {
            return correction(ctx, 2, selector, summon);
        };
        cells.push(validation::Cell {
            x: x as i16,
            y: y as i16,
            z: z as i16,
        });
        if cells.len() >= 36 {
            return correction(ctx, 102, selector, summon);
        }
    }
    if validation::handler_counts(types.len(), declared as usize, cells.len()).is_err() {
        return correction(ctx, 102, selector, summon);
    }
    if zone.is_none() {
        return correction(ctx, 2, selector, summon);
    }
    if !matches!(kind, 1 | 2) {
        return Outcome::default();
    }
    if validation::initializer_limits(
        zone,
        zone,
        kind,
        Some(&cells),
        u32::from(duration),
        types.len(),
    )
    .is_err()
    {
        return correction(ctx, 2, selector, summon);
    }
    // TODO ❓ PC duration-history initialization precedes action start. Do not
    // run records/speed/height out of order or treat this prefix as approval.
    Outcome::unsupported("TODO ❓ movement duration history, action setup, speed/height/transition integration and continuations")
}
fn correction(ctx: &Context<'_>, reason: u16, selector: u8, summon: bool) -> Outcome {
    let (position, yaw) = if summon {
        ctx.pc()
            .summon
            .as_ref()
            .map(|s| (s.position, s.yaw))
            .unwrap_or((ctx.pc().position, ctx.pc().yaw))
    } else {
        (ctx.pc().position, ctx.pc().yaw)
    };
    let mut body = reason.to_le_bytes().to_vec();
    body.push(selector);
    for value in [position.x as u16, position.y as u16, position.z as u16, yaw] {
        body.extend(value.to_le_bytes());
    }
    Outcome::reply(65, body)
}

/// parity: 66 11-movement-handlers.md#66--g_turnreqcmd--partial partial
pub fn handle_turn(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    let mut r = Reader(req);
    let (Some(selector), Some(yaw)) = (r.byte(), r.short()) else {
        return Outcome::unsupported("TODO ❓ unchecked Turn decode");
    };
    let summon = match validation::selects_summon(
        selector,
        ctx.pc().summon.as_ref().map(|s| s.control_state),
    ) {
        Ok(value) => value,
        Err(_) => {
            return Outcome::unsupported(
                "TODO ❓ Turn wrapper missing-creature reason/selector, 0x14038B990",
            )
        }
    };
    let zone = if summon {
        ctx.pc().summon.as_ref().and_then(|s| s.zone)
    } else {
        ctx.pc().zone
    };
    if zone.is_none() {
        let mut out = Outcome::reason(67, 2);
        out.replies[0].body.push(selector);
        return out;
    }
    if yaw > 359 {
        return Outcome::unsupported("TODO ❓ Turn wrapper yaw failure arguments");
    }
    // TODO ❓ Queue failure arguments and global yaw bounds are not recovered.
    match ctx.services.queue(66) {
        Gate::Pass => Outcome { changes: vec![StateChange::Queued(66)], ..Outcome::unsupported("TODO ❓ TurnTask yaw bounds / property and movement broadcasts; deferred failure 67/168") },
        _ => Outcome::unsupported("TODO ❓ Turn task creation/queue failure arguments"),
    }
}

/// parity: 99 11-movement-handlers.md#99--g_activatedteleportlistreqcmd--partial partial
pub fn handle_activated_teleport_list(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if ctx.pc().zone.is_none() {
        return Outcome::reason(101, 2);
    }
    ctx.queue(
        99,
        101,
        2,
        "TODO ❓ ActivatedTeleport property metadata / packed body / unavailable property contract",
    )
}

/// parity: 102 11-movement-handlers.md#102--g_detectteleportswitchreqcmd--partial partial
pub fn handle_detect_teleport_switch(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if ctx.pc().zone.is_none() {
        return Outcome::reason(104, 2);
    }
    let Some(id) = Reader(req).short() else {
        return Outcome::unsupported("TODO ❓ unchecked teleport-switch decode");
    };
    if ctx.pc().native_kind != 1 {
        return Outcome::reason(104, 2);
    }
    let mut out = ctx.queue(102, 104, 2, "TODO ❓ activation metadata/set/persistence");
    if !out.changes.contains(&StateChange::Queued(102)) {
        return out;
    }
    // The adapter keys the switch map by the current runtime zone. It must
    // resolve the native (geoZoneId, unsigned switchId) table key first.
    let switch = ctx
        .pc()
        .zone
        .and_then(|zone| ctx.world.switches.get(&(zone, id)));
    let reason = match switch {
        None => Some(1128),
        Some(s)
            if ctx.pc().position.squared_distance(s.position)
                > i64::from(s.radius) * i64::from(s.radius) =>
        {
            Some(1129)
        }
        Some(s) => match ctx.world.teleports.get(&s.teleport_key) {
            None => Some(1128),
            Some(t) if t.disabled => Some(1128),
            Some(_) => None,
        },
    };
    if let Some(reason) = reason {
        out = Outcome::reason(104, reason);
        out.changes.push(StateChange::Queued(102));
    }
    // TODO ❓ Property::Set and table bit-index bounds remain unknown. No
    // activation bit, notify packet or persistence send before that boundary.
    out
}

/// parity: 145 11-movement-handlers.md#145--g_returntobasecampreqcmd--partial partial
pub fn handle_return_to_base_camp(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if ctx.pc().zone.is_none() {
        return Outcome::reason(146, 2);
    }
    ctx.queue(
        145,
        146,
        2,
        "TODO ❓ Pc::ReturnToBaseCamp, 0x1405E6EE0; false -> 146/284",
    )
}

/// parity: 296 11-movement-handlers.md#296--g_enterportalreqcmd--partial partial
pub fn handle_enter_portal(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if ctx.pc().zone.is_none() || ctx.pc().native_kind != 1 {
        return Outcome::reason(297, 2);
    }
    let out = ctx.queue(296, 297, 2, "TODO ❓ EnterPortalTask");
    if !out.changes.contains(&StateChange::Queued(296)) {
        return out;
    }
    let Some(zone) = ctx.pc().zone.and_then(|id| ctx.world.zones.get(&id)) else {
        return out;
    };
    let mut result = if zone.environment_index <= 0 || zone.portal_subtype != Some(7) {
        Outcome::reason(297, 1152)
    } else {
        match zone.life_state {
            0 => Outcome::unsupported(
                "TODO ❓ portal helper / reason converter / destination, 0x1405A96F0",
            ),
            1 | 3 => Outcome::reason(297, 1155),
            2 => Outcome::unsupported(
                "TODO ❓ portal life-state 2 virtual +128 mutation precedes 297/1155",
            ),
            _ => Outcome::reason(297, 2),
        }
    };
    result.changes.push(StateChange::Queued(296));
    result
}

/// parity: 298 11-movement-handlers.md#298--g_turnbackfromportalreqcmd--partial partial
pub fn handle_turn_back_from_portal(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if ctx.pc().zone.is_none() {
        return Outcome::reason(299, 2);
    }
    if req.len() < 8 {
        return Outcome::unsupported("TODO ❓ unchecked portal-return pose decode");
    }
    ctx.queue(298, 299, 2, "TODO ❓ WarpAction class 33 terrain/range checks and continuations; no coordinate authorization")
}

/// parity: 648 11-movement-handlers.md#648--g_returntogamereqcmd--partial partial
pub fn handle_return_to_game(ctx: &mut Context<'_>, _: &[u8]) -> Outcome {
    if ctx.world.arena {
        return Outcome::reason(650, 5);
    }
    if ctx.pc().arena_state != 4 {
        return Outcome::reason(650, 2);
    }
    let pc = ctx.session.pc.as_mut().expect("PC dispatch");
    pc.arena_state = 1;
    pc.arena_qwords = [0; 2];
    Outcome {
        changes: vec![StateChange::ArenaReturned],
        daemon_requests: vec![DaemonRequest::ReturnToGame {
            routing: pc.routing,
            db_id: pc.db_id,
            object_id: pc.object_id,
        }],
        ..Outcome::unsupported("TODO ❓ return-to-game backend response / late rollback")
    }
}

/// parity: 1210 11-movement-handlers.md#1210--g_clientrunreqcmd--partial partial
pub fn handle_client_run(_: &mut Context<'_>, req: &[u8]) -> Outcome {
    if req.len() < 2 {
        return Outcome::log("ClientRun decode failure");
    }
    Outcome::unsupported(
        "TODO ❓ ClientRun report helper effects / byte meanings, 0x14060D810 / 0x140368150",
    )
}

/// parity: 1219 11-movement-handlers.md#1219--g_airdashreqcmd--partial partial
pub fn handle_air_dash(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if ctx.pc().zone.is_none() {
        return Outcome::reason(1220, 2);
    }
    let Some(id) = Reader(req).quad() else {
        return Outcome::unsupported("TODO ❓ unchecked AirDash q value");
    };
    if ctx.pc().native_kind != 1 {
        return Outcome::reason(1220, 2);
    }
    let mut out = ctx.queue(
        1219,
        1220,
        2,
        "TODO ❓ AirDash start/effect helpers and cleanup, 0x1404808A0",
    );
    if !out.changes.contains(&StateChange::Queued(1219)) {
        return out;
    }
    let index = id as u32 as i32;
    let zone = ctx.pc().zone.expect("zone checked");
    let env = ctx.world.air_dash.get(&(zone, index));
    let reason = if !(1..=255).contains(&index) {
        Some(169)
    } else {
        match env {
            None => Some(169),
            Some(e) if !matches!(e.subtype, 11 | 12) => Some(169),
            Some(e) => {
                let Some(zone_data) = ctx.world.zones.get(&zone) else {
                    out.status = Status::Unsupported("TODO ❓ missing server environment index");
                    return out;
                };
                if e.required_environment != zone_data.environment_index {
                    Some(169)
                } else if !e.enabled {
                    Some(171)
                } else if e.helper_65002 == Gate::Unknown {
                    out.status = Status::Unsupported("TODO ❓ AirDash helper 65002, 0x14059A9C0");
                    return out;
                } else if e.helper_65002 == Gate::Fail {
                    Some(170)
                } else if e.faction_requirement != 0
                    && e.faction_requirement != i64::from(e.pc_faction)
                {
                    Some(174)
                } else if e.subtype == 11
                    && e.requirement_92 > 0
                    && e.pc_value_552 < e.requirement_92
                {
                    Some(172)
                } else if e.subtype == 11 && e.zone_helper == Some(Gate::Unknown) {
                    out.status = Status::Unsupported("TODO ❓ AirDash zone helper, 0x1401035B0");
                    return out;
                } else if e.subtype == 11 && e.zone_helper == Some(Gate::Fail) {
                    Some(173)
                } else {
                    None
                }
            }
        }
    };
    if let Some(reason) = reason {
        out = Outcome::reason(1220, reason);
        out.changes.push(StateChange::Queued(1219));
    }
    out
}

/// parity: 1223 11-movement-handlers.md#1223--g_useteleportreqcmd--partial partial
pub fn handle_use_teleport(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if ctx.world.arena || super::session::arena_blocked(ctx.pc()) {
        return teleport_failure(5);
    }
    let mut r = Reader(req);
    let (Some(_start), Some(_dest)) = (r.short(), r.short()) else {
        return Outcome::unsupported("TODO ❓ unchecked teleport IDs / nonpositive activation IDs");
    };
    if ctx.pc().zone.is_none() {
        return teleport_failure(2);
    }
    if ctx.pc().achievement_reward || ctx.pc().jumping_ticket {
        return teleport_failure(1144);
    }
    if ctx.pc().instant_payment {
        return teleport_failure(1145);
    }
    if ctx.pc().native_kind != 1 {
        return teleport_failure(2);
    }
    let mut out = ctx.queue(
        1223,
        1225,
        2,
        "TODO ❓ Teleport action gates, event rules, tables, start position, cost and LeaveAction",
    );
    if let Some(reply) = out.replies.first_mut() {
        reply.body.extend(0u64.to_le_bytes());
    }
    out
}
fn teleport_failure(reason: u16) -> Outcome {
    let mut out = Outcome::reason(1225, reason);
    out.replies[0].body.extend(0u64.to_le_bytes());
    out
}

/// parity: 1239 11-movement-handlers.md#1239--g_usecaveescapereqcmd--partial partial
pub fn handle_use_cave_escape(ctx: &mut Context<'_>, req: &[u8]) -> Outcome {
    if ctx.pc().zone.is_none() {
        return Outcome::reason(1241, 1);
    }
    let Some(_position) = Reader(req).short() else {
        return Outcome::unsupported("TODO ❓ unchecked cave-escape inventory position");
    };
    if ctx.pc().achievement_reward || ctx.pc().jumping_ticket {
        return Outcome::reason(1241, 254);
    }
    if ctx.pc().instant_payment {
        return Outcome::reason(1241, 255);
    }
    if ctx.pc().native_kind != 1 {
        return Outcome::reason(1241, 1);
    }
    // TODO ❓ Do not send start or consume before unknown item checks/busy flag.
    ctx.queue(1239, 1241, 1, "TODO ❓ cave-escape item validation/reservation, disposal position register, destination and LeaveAction")
}
