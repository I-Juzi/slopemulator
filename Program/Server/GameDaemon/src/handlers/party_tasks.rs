//! Explicit queued-task continuations for doc 14. No removal/auction/DB policy
//! is inferred from a task name. Opaque managers require a server-owned adapter.
use super::{party::*, *};

/// Execute only after the adapter dequeues the task, with its caster as the PC.
/// The wrapper itself never calls this function. Adapter ownership is exclusive.
pub fn execute_party_task(ctx: &mut Context<'_>, task: &PartyTask) -> Outcome {
    let Some(pc) = ctx.session.pc.clone() else {
        return boundary("❓ task caster PC unavailable");
    };
    if pc.object_id != task.caster {
        return boundary("❓ task caster identity mismatch");
    }
    match task.kind {
        PartyTaskKind::TeamAuthority { authority } => {
            if pc.party_id == 0 {
                return Outcome::reason(173, 781);
            }
            let Some(state) = ctx.services.party_state() else {
                return boundary("❓ PARTY state adapter");
            };
            change_team_authority(&pc, state, authority)
        }
        PartyTaskKind::PartyAuthority { authority } => {
            if pc.party_id == 0 {
                return Outcome::reason(184, 796);
            }
            let Some(state) = ctx.services.party_state() else {
                return boundary("❓ PARTY state adapter");
            };
            change_party_authority(&pc, state, authority)
        }
        PartyTaskKind::Pickup {
            rule,
            grade,
            target,
            money,
        } => {
            if pc.party_id == 0 {
                return Outcome::reason(178, 787);
            }
            let Some(state) = ctx.services.party_state() else {
                return boundary("❓ PARTY state adapter");
            };
            change_pickup(
                &pc,
                state,
                &ctx.world.party_config,
                rule,
                grade,
                target,
                money,
            )
        }
        PartyTaskKind::Difficulty { difficulty } => {
            if pc.party_id == 0 {
                return byte_reply(181, 789);
            }
            change_difficulty(ctx, &pc, difficulty, None)
        }
        PartyTaskKind::LeaveParty => {
            match suppress_leave(ctx, task, true) {
                Ok(true) => return Outcome::default(),
                Err(why) => return boundary(why),
                Ok(false) => {}
            }
            opaque(
                ctx,
                PartyOperation::LeaveParty,
                "❓ party removal manager 0x1401D55D0",
            )
        }
        PartyTaskKind::LeaveTeam { .. } => {
            match suppress_leave(ctx, task, false) {
                Ok(true) => return Outcome::default(),
                Err(why) => return boundary(why),
                Ok(false) => {}
            }
            match ctx.services.party_operation(PartyOperation::LeaveTeam) {
                None => boundary("❓ team removal manager 0x1401D5F50"),
                Some(0) => Outcome::default(),
                Some(reason) => {
                    let mut out = Outcome::reason(244, reason as u16);
                    out.replies[0].body.push(2);
                    out
                }
            }
        }
        PartyTaskKind::DelegateParty { target } => delegate_party(ctx, &pc, target),
        PartyTaskKind::BanishTeam { party, slot } => {
            if pc.party_id == 0 {
                return Outcome::reason(246, 781);
            }
            opaque(
                ctx,
                PartyOperation::BanishTeam { party, slot },
                "❓ team ban manager 0x1401D7FD0",
            )
        }
        PartyTaskKind::MoveMember {
            source,
            destination,
        } => {
            if pc.party_id == 0 {
                return Outcome::reason(249, 880);
            }
            opaque(
                ctx,
                PartyOperation::MoveMember {
                    source,
                    destination,
                },
                "❓ member move manager 0x1401D8C50",
            )
        }
        PartyTaskKind::DelegateTeam { target } | PartyTaskKind::NominateLeader { target } => {
            let nominate = matches!(task.kind, PartyTaskKind::NominateLeader { .. });
            let (command, missing_party, missing_pc) = if nominate {
                (255, 891, 890)
            } else {
                (252, 887, 889)
            };
            if pc.party_id == 0 {
                return Outcome::reason(command, missing_party);
            }
            let Some(state) = ctx.services.party_state() else {
                return boundary("❓ PARTY state adapter");
            };
            let Some(peer) = state.by_id(target).map(|p| p.pc.clone()) else {
                return Outcome::reason(command, missing_pc);
            };
            let operation = if nominate {
                PartyOperation::NominateLeader {
                    target_db: peer.db_id,
                    target_runtime: peer.object_id,
                }
            } else {
                PartyOperation::DelegateTeam {
                    target_db: peer.db_id,
                    target_runtime: peer.object_id,
                }
            };
            opaque(
                ctx,
                operation,
                "❓ leader manager mutation/notifications/backend",
            )
        }
        PartyTaskKind::TeamDifficulty {
            selector,
            difficulty,
        } => {
            if pc.party_id == 0 {
                return Outcome::reason(261, 897);
            }
            change_difficulty(ctx, &pc, difficulty, Some(selector))
        }
        PartyTaskKind::ClearMembers => {
            if pc.party_id == 0 {
                return Outcome::reason(175, 2);
            }
            opaque(
                ctx,
                PartyOperation::ClearMembers,
                "❓ expired-member manager 0x1401DD540",
            )
        }
        PartyTaskKind::UpdatePcInfo { team, party } => update_pc_info(ctx, &pc, team, party),
        PartyTaskKind::BanishParty { slot } => {
            // Build 381.8867, 0x1402B4840: no party is 173/781.
            if pc.party_id == 0 {
                return Outcome::reason(173, 781);
            }
            opaque(
                ctx,
                PartyOperation::BanishParty { slot },
                "❓ party ban manager 0x1401D71D0 removal/leader/notifications",
            )
        }
        PartyTaskKind::InviteParty { .. } | PartyTaskKind::InviteTeam { .. } => {
            boundary("❓ invitation task execution/timeout/authorization/backend")
        }
        PartyTaskKind::JoinParty { .. } | PartyTaskKind::JoinTeam { .. } => {
            boundary("❓ join task authorization/insertion/sync/backend")
        }
        PartyTaskKind::Reject {
            group,
            invitation,
            correlation,
            kind,
        } => reject_invitation(ctx, &pc, group, invitation, correlation, kind),
        PartyTaskKind::SuggestParty { requester, options } => {
            suggest_party_task(ctx, &pc, requester, options)
        }
        PartyTaskKind::Vote { .. } => {
            // Build 381.8867, 0x1402B5540: these gates precede the clock read.
            if pc.party_id == 0 {
                return Outcome::reason(219, 812);
            }
            if pc.team_member && pc.team_id != 0 {
                return Outcome::reason(219, 814);
            }
            boundary("❓ party-ban vote clock 0x1402B5540 and tally/removal continuation")
        }
        PartyTaskKind::Auction { .. } => {
            boundary("❓ auction task identity/authorization/item/money/persistence")
        }
    }
}

fn reject_invitation(
    ctx: &mut Context<'_>,
    caster: &Pc,
    group: u8,
    inviter: u64,
    correlation: u8,
    kind: u8,
) -> Outcome {
    let Some(state) = ctx.services.party_state() else {
        return boundary("❓ PARTY state adapter");
    };
    let Some(inviter) = state.by_id(inviter).map(|peer| peer.pc.clone()) else {
        return Outcome::log("party rejection inviter missing");
    };
    let Some(boxes) = state.invitation_boxes.as_mut() else {
        return boundary("❓ party invitation-card box adapter");
    };
    let cards = boxes.entry(caster.object_id).or_default();
    let mut out = Outcome::default();
    let kinds = if group == 1 { 1..=1 } else { 2..=4 };
    let mut removed = false;
    for card_kind in kinds {
        if let Some(index) = cards.iter().position(|card| {
            card.kind == card_kind
                && card.inviter_db == inviter.db_id
                && card.inviter_runtime == inviter.object_id
        }) {
            out.party_effects.push(PartyEffect::InvitationRemoved {
                owner: caster.object_id,
                invitation: cards.remove(index),
            });
            removed = true;
        }
    }
    if removed {
        let mut body = caster.object_id.to_le_bytes().to_vec();
        string_body(&mut body, &caster.name);
        if group == 1 {
            body.push(correlation);
        }
        body.push(kind);
        send(
            &mut out,
            inviter.object_id,
            Message {
                command: if group == 1 { 186 } else { 264 },
                body,
            },
        );
    }
    if ctx.world.party_config.trace_report || ctx.world.party_config.rejection_aux_report {
        out.party_effects.push(PartyEffect::Boundary(
            "❓ rejection report sinks 0x140203710 / daemon +2728",
        ));
        out.status = Status::Unsupported("❓ rejection report adapters");
    }
    out
}

fn suggest_party_task(
    ctx: &mut Context<'_>,
    caster: &Pc,
    requester: u64,
    options: [u8; 6],
) -> Outcome {
    let Some(state) = ctx.services.party_state() else {
        return boundary("❓ PARTY state adapter");
    };
    let Some(requester) = state.by_id(requester).map(|peer| peer.pc.clone()) else {
        return Outcome::log("party suggestion requester missing");
    };
    let fail = || {
        let message = suggestion_failure(
            150,
            740,
            None,
            caster.object_id,
            &caster.name,
            &options[..2],
        )
        .replies
        .remove(0);
        let mut out = Outcome::default();
        send(&mut out, requester.object_id, message);
        out
    };
    let recipient = if caster.party_id == 0 {
        caster.clone()
    } else {
        let Some(location) = state.resolve(caster.db_id, caster.object_id) else {
            return Outcome::log("party suggestion team missing");
        };
        let team = state.teams[location.team].as_ref().unwrap();
        let party = team.parties[location.party].as_ref().unwrap();
        let (selected, party_route, direct) = if !ctx.world.party_config.team_suggestion_routing {
            let selected = match party.invite_authority {
                0 => party.members[0].runtime_id,
                1 => party.members[location.member].runtime_id,
                _ => return fail(),
            };
            (selected, true, false)
        } else {
            let permitted = if team.mode == 1 {
                team.allows_invite(2, location.party as u8, caster.db_id, caster.object_id)
            } else {
                match party.invite_authority {
                    0 => location.member == 0,
                    1 => true,
                    _ => false,
                }
            };
            (
                if permitted {
                    caster.object_id
                } else {
                    team.leader_runtime_id
                },
                false,
                permitted,
            )
        };
        if selected == 0 || (party_route && selected == requester.object_id) {
            return fail();
        }
        let recipient = if direct {
            caster.clone()
        } else {
            let Some(peer) = state.by_id(selected) else {
                return fail();
            };
            peer.pc.clone()
        };
        // Direct team-aware notification to the caster has no second arena gate.
        if !direct
            && ctx.world.party_config.arena_enabled
            && !ctx.world.arena
            && session::arena_blocked(&recipient)
        {
            return fail();
        }
        recipient
    };
    let mut out = Outcome::default();
    send(
        &mut out,
        recipient.object_id,
        suggestion_notify(149, None, &requester.name, &options),
    );
    send(
        &mut out,
        requester.object_id,
        suggestion_ack(
            148,
            None,
            recipient.object_id,
            &recipient.name,
            &options[..2],
        ),
    );
    out
}

fn byte_reply(command: u16, reason: u16) -> Outcome {
    Outcome::reply(command, vec![reason as u8])
}
fn opaque(ctx: &mut Context<'_>, operation: PartyOperation, why: &'static str) -> Outcome {
    // ❓ Manager effects are not represented by a fabricated success result.
    let _ = ctx.services.party_operation(operation);
    boundary(why)
}
fn suppress_leave(ctx: &Context<'_>, task: &PartyTask, party: bool) -> Result<bool, &'static str> {
    if ctx.world.arena && ctx.world.party_config.arena_task_mode == 2 {
        if let Some(id) = ctx.pc().zone {
            let zone = ctx
                .world
                .zones
                .get(&id)
                .ok_or("❓ current task PC zone metadata")?;
            if zone.party_subtype == 164 {
                return Ok(true);
            }
        }
    }
    if party {
        let zone = ctx
            .world
            .zones
            .get(&task.zone)
            .ok_or("❓ leave task zone attraction metadata")?;
        if zone.attraction_kind == Some(10) {
            return Ok(true);
        }
    }
    Ok(false)
}
fn fanout(out: &mut Outcome, recipients: Vec<u64>, message: Message) {
    for to in recipients {
        send(out, to, message.clone());
    }
}
fn change_team_authority(pc: &Pc, state: &mut PartyState, authority: u8) -> Outcome {
    let Some(location) = state.resolve(pc.db_id, pc.object_id) else {
        return Outcome::reason(258, 895);
    };
    let team = state.teams[location.team].as_ref().unwrap();
    if team.leader_db_id != pc.db_id || team.leader_runtime_id != pc.object_id {
        return Outcome::reason(258, 895);
    }
    let recipients = state.team_recipients(team);
    state.teams[location.team]
        .as_mut()
        .unwrap()
        .invite_authority = u32::from(authority);
    let mut out = Outcome::default();
    out.party_effects.push(PartyEffect::TeamAuthorityChanged {
        team: location.team,
        authority,
    });
    fanout(
        &mut out,
        recipients,
        Message {
            command: 274,
            body: vec![authority],
        },
    );
    out
}
fn change_party_authority(pc: &Pc, state: &mut PartyState, authority: u8) -> Outcome {
    let Some(location) = state.resolve(pc.db_id, pc.object_id) else {
        return Outcome::reason(184, 796);
    };
    if location.member != 0 {
        return Outcome::reason(184, 797);
    }
    let party = state.teams[location.team].as_mut().unwrap().parties[location.party]
        .as_mut()
        .unwrap();
    if party.invite_authority == u32::from(authority) {
        return Outcome::reason(184, 798);
    }
    party.invite_authority = u32::from(authority);
    let party = state.teams[location.team].as_ref().unwrap().parties[location.party]
        .as_ref()
        .unwrap();
    let recipients = state.recipients(party);
    let mut out = Outcome::default();
    out.party_effects.push(PartyEffect::PartyAuthorityChanged {
        location,
        authority,
    });
    fanout(
        &mut out,
        recipients,
        Message {
            command: 183,
            body: vec![authority],
        },
    );
    out
}
fn change_pickup(
    pc: &Pc,
    state: &mut PartyState,
    config: &PartyConfig,
    mut rule: u8,
    grade: u8,
    target: u64,
    money: u64,
) -> Outcome {
    let Some(location) = state.resolve(pc.db_id, pc.object_id) else {
        return Outcome::reason(178, 786);
    };
    let party = state.teams[location.team].as_mut().unwrap().parties[location.party]
        .as_mut()
        .unwrap();
    if location.member != 0 || (grade as i8) <= 0 || i32::from(grade as i8) > config.maximum_grade {
        return Outcome::reason(178, 786);
    }
    if target != 0
        && !party
            .members
            .iter()
            .any(|m| m.runtime_id == target && m.state == 3)
    {
        return Outcome::reason(178, 786);
    }
    let old = (party.pickup_rule as u8, party.grade, party.money);
    let report_pc = party.members[location.member].runtime_id;
    let clamped = rule == 3 && config.rule_three_mode != 2;
    if clamped {
        rule = 0;
    }
    party.pickup_rule = i32::from(rule as i8);
    party.grade = grade;
    party.designated_id = target;
    party.money = money;
    let mut body = vec![rule, grade];
    body.extend(target.to_le_bytes());
    body.extend(money.to_le_bytes());
    let party = state.teams[location.team].as_ref().unwrap().parties[location.party]
        .as_ref()
        .unwrap();
    let recipients = state.recipients(party);
    let mut out = Outcome::default();
    if clamped {
        out.party_effects.push(PartyEffect::AssertionReport);
    }
    out.party_effects
        .push(PartyEffect::PickupChanged { location });
    fanout(&mut out, recipients, Message { command: 177, body });
    if config.trace_report && old != (rule, grade, money) && state.by_id(report_pc).is_some() {
        out.party_effects.push(PartyEffect::Boundary(
            "❓ pickup LogArchive report 0x140203F80, operation 1",
        ));
        out.status = Status::Unsupported("❓ pickup LogArchive report adapter");
    }
    out
}
fn change_difficulty(
    ctx: &mut Context<'_>,
    pc: &Pc,
    difficulty: u8,
    selector: Option<u8>,
) -> Outcome {
    let config = &ctx.world.party_config;
    let Some(state) = ctx.services.party_state() else {
        return boundary("❓ PARTY state adapter");
    };
    let Some(mut location) = state.resolve(pc.db_id, pc.object_id) else {
        return if selector.is_some() {
            Outcome::reason(261, 897)
        } else {
            byte_reply(181, 2)
        };
    };
    if let Some(selector) = selector {
        let team = state.teams[location.team].as_ref().unwrap();
        if team.leader_db_id != pc.db_id || team.leader_runtime_id != pc.object_id {
            return Outcome::reason(261, 897);
        }
        if (selector as i8) < 0
            || team
                .parties
                .get(usize::from(selector))
                .and_then(|p| p.as_ref())
                .is_none()
        {
            return Outcome::reason(261, 2);
        }
        location.party = usize::from(selector);
        location.member = 0;
    }
    let party = state.teams[location.team].as_mut().unwrap().parties[location.party]
        .as_mut()
        .unwrap();
    let maximum = if (1..=3).contains(&difficulty) {
        i32::from(*config.difficulty_maximum.get(&difficulty).unwrap_or(&0))
    } else {
        0
    };
    let reason = if location.member != 0 {
        790
    } else if party.difficulty == difficulty {
        791
    } else if maximum < 0 || (maximum as usize) < party.members.len() {
        793
    } else if party.difficulty_blockers.iter().any(|v| *v > 0) {
        794
    } else {
        0
    };
    if selector.is_some() && reason != 0 {
        return Outcome::reason(261, reason);
    }
    let mut out = Outcome::default();
    if reason == 0 {
        party.difficulty = difficulty;
        out.party_effects.push(PartyEffect::DifficultyChanged {
            location,
            difficulty,
        });
    }
    let team = state.teams[location.team].as_ref().unwrap();
    let party = team.parties[location.party].as_ref().unwrap();
    let (team_id, party_id, mode, current) = (
        team.runtime_id,
        party.runtime_id,
        team.mode,
        party.difficulty,
    );
    let recipients = state.team_recipients(team);
    let tasks: Vec<_> = if reason == 0 {
        party
            .members
            .iter()
            .enumerate()
            .filter(|(index, member)| member.state == 3 && (selector.is_some() || *index != 0))
            .map(|(_, member)| state.by_id(member.runtime_id).map(|peer| peer.pc.clone()))
            .collect()
    } else {
        vec![]
    };
    let report = reason == 0
        && config.trace_report
        && party
            .members
            .first()
            .and_then(|m| state.by_id(m.runtime_id))
            .is_some();
    for peer in tasks {
        // 0x14021EDE0 logs each failed lookup and continues with later members.
        let Some(peer) = peer else {
            out.logs.push("party metadata task PC missing");
            continue;
        };
        let Some(zone) = peer.zone else {
            out.logs.push("party metadata task zone missing");
            continue;
        };
        if peer.native_kind != 1 {
            out.logs.push("party metadata task caster is not PC");
            out.party_effects.push(PartyEffect::AssertionReport);
            continue;
        }
        let task = PartyTask {
            command: if selector.is_some() { 259 } else { 179 },
            caster: peer.object_id,
            zone,
            kind: PartyTaskKind::UpdatePcInfo {
                team: team_id,
                party: party_id,
            },
        };
        match ctx.services.queue_party(&task) {
            Gate::Pass => out.changes.push(StateChange::PartyTaskQueued(task)),
            Gate::Fail => out.logs.push("party metadata task dispatch failed"),
            Gate::Unknown => {
                out.party_effects.push(PartyEffect::Boundary(
                    "❓ party metadata task dispatch adapter",
                ));
                out.status = Status::Unsupported("❓ party metadata task dispatch adapter");
                return out;
            }
        }
    }
    if report {
        out.party_effects.push(PartyEffect::Boundary(
            "❓ difficulty LogArchive report 0x140203F80, operation 2",
        ));
        out.status = Status::Unsupported("❓ difficulty LogArchive report adapter");
        return out;
    }
    if let Some(selector) = selector {
        fanout(
            &mut out,
            recipients,
            Message {
                command: 275,
                body: vec![selector, difficulty],
            },
        );
        // Descriptor hc consumes only the selector. The extra native argument is unused.
        self_send(
            &mut out,
            pc,
            Message {
                command: 260,
                body: vec![selector],
            },
        );
    } else {
        if reason == 0 {
            self_send(
                &mut out,
                pc,
                Message {
                    command: 193,
                    body: vec![difficulty],
                },
            );
            self_send(
                &mut out,
                pc,
                Message {
                    command: 180,
                    body: vec![difficulty],
                },
            );
        } else {
            self_send(&mut out, pc, byte_failure(181, reason));
        }
        apply_pc_info(ctx, &mut out, team_id, party_id, mode, current);
        if mode != 0 {
            fanout(
                &mut out,
                recipients,
                Message {
                    command: 275,
                    body: vec![location.party as u8, current],
                },
            );
        }
    }
    out
}

fn apply_pc_info(
    ctx: &mut Context<'_>,
    out: &mut Outcome,
    team: u64,
    party: u64,
    mode: u8,
    difficulty: u8,
) {
    let pc = ctx.session.pc.as_mut().unwrap();
    if pc.team_id == team && pc.party_id == party {
        pc.party_difficulty = difficulty;
        pc.party_team_mode = mode;
        pc.team_member = mode != 0;
        out.party_effects.push(PartyEffect::PcInfoChanged {
            runtime: pc.object_id,
            team_mode: mode,
            difficulty,
        });
    } else {
        out.logs.push("party metadata identity mismatch");
    }
}

fn update_pc_info(
    ctx: &mut Context<'_>,
    pc: &Pc,
    expected_team: u64,
    expected_party: u64,
) -> Outcome {
    if pc.party_id == 0 {
        return Outcome::log("party metadata PC has no party");
    }
    let Some(state) = ctx.services.party_state() else {
        return boundary("❓ PARTY state adapter");
    };
    let Some(location) = state.resolve(pc.db_id, pc.object_id) else {
        return Outcome::log("party metadata team missing");
    };
    let team = state.teams[location.team].as_ref().unwrap();
    let party = team.parties[location.party].as_ref().unwrap();
    if team.runtime_id != expected_team || party.runtime_id != expected_party {
        return Outcome::log("party metadata task team changed");
    }
    let (mode, difficulty) = (team.mode, party.difficulty);
    let mut out = Outcome::default();
    apply_pc_info(
        ctx,
        &mut out,
        expected_team,
        expected_party,
        mode,
        difficulty,
    );
    // The native comparison uses the old byte even if the guarded write failed.
    if pc.party_difficulty != difficulty {
        self_send(
            &mut out,
            pc,
            Message {
                command: 193,
                body: vec![difficulty],
            },
        );
    }
    out
}
fn delegate_party(ctx: &mut Context<'_>, pc: &Pc, target: u64) -> Outcome {
    let Some(state) = ctx.services.party_state() else {
        return boundary("❓ PARTY state adapter");
    };
    let Some(peer) = state.by_id(target).map(|p| p.pc.clone()) else {
        return Outcome::reason(171, 779);
    };
    if pc.party_id == 0 {
        return Outcome::reason(171, 779);
    }
    let Some(location) = state.resolve(pc.db_id, pc.object_id) else {
        return Outcome::reason(171, 779);
    };
    let team = state.teams[location.team].as_ref().unwrap();
    let was_team_leader = team.leader_db_id == pc.db_id && team.leader_runtime_id == pc.object_id;
    if !team.parties[location.party]
        .as_ref()
        .unwrap()
        .members
        .iter()
        .any(|m| m.db_id == peer.db_id && m.runtime_id == peer.object_id && m.state == 3)
    {
        return Outcome::reason(171, 779);
    }
    match ctx.services.party_operation(PartyOperation::DelegateParty {
        location,
        target_db: peer.db_id,
        target_runtime: peer.object_id,
    }) {
        None => return boundary("❓ party-leader mutation 0x1401EB9C0"),
        Some(0) => return Outcome::reason(171, 779),
        Some(_) => {}
    }
    if was_team_leader {
        match ctx
            .services
            .party_operation(PartyOperation::TransferTeamLeader {
                team: location.team,
                target_db: peer.db_id,
                target_runtime: peer.object_id,
            }) {
            None => return boundary("❓ team-leader transfer 0x1401F3D30; no inferred rollback"),
            Some(0) => return Outcome::reason(171, 886),
            Some(_) => {}
        }
    }
    let Some(state) = ctx.services.party_state() else {
        return boundary("❓ PARTY state adapter");
    };
    let Some(team) = state.teams.get(location.team).and_then(|t| t.as_ref()) else {
        return boundary("❓ team missing after leader helper");
    };
    let mut out = boundary("❓ leader helper notifications/backend and rollback");
    if team.mode == 1 {
        let id = if was_team_leader {
            team.leader_runtime_id
        } else {
            peer.object_id
        };
        fanout(
            &mut out,
            state.team_recipients(team),
            Message {
                command: if was_team_leader { 272 } else { 273 },
                body: id.to_le_bytes().to_vec(),
            },
        );
    }
    out
}
