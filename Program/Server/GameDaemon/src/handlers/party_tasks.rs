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
            let Some(state) = ctx.services.party_state() else {
                return boundary("❓ PARTY state adapter");
            };
            change_difficulty(&pc, state, &ctx.world.party_config, difficulty)
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
            opaque(
                ctx,
                PartyOperation::TeamDifficulty {
                    selector,
                    difficulty,
                },
                "❓ team party difficulty manager 0x1401E1FC0",
            )
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
        PartyTaskKind::BanishParty { .. } => {
            boundary("❓ party ban task role/slot/removal/leader/notifications")
        }
        PartyTaskKind::InviteParty { .. } | PartyTaskKind::InviteTeam { .. } => {
            boundary("❓ invitation task execution/timeout/authorization/backend")
        }
        PartyTaskKind::JoinParty { .. } | PartyTaskKind::JoinTeam { .. } => {
            boundary("❓ join task authorization/insertion/sync/backend")
        }
        PartyTaskKind::Reject { .. } => boundary("❓ rejection match/notification/timeout task"),
        PartyTaskKind::SuggestParty { .. } => {
            boundary("❓ party suggestion task execution/notification policy")
        }
        PartyTaskKind::Vote { .. } => boundary("❓ party-ban vote task state/role/tally/timing"),
        PartyTaskKind::Auction { .. } => {
            boundary("❓ auction task identity/authorization/item/money/persistence")
        }
    }
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
    if rule == 3 && config.rule_three_mode != 2 {
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
    out.party_effects
        .push(PartyEffect::PickupChanged { location });
    fanout(&mut out, recipients, Message { command: 177, body });
    out.party_effects.push(PartyEffect::Boundary(
        "❓ optional pickup report/downstream effects",
    ));
    out.status =
        Status::Unsupported("❓ pickup field/config names and optional report/downstream effects");
    out
}
fn change_difficulty(
    pc: &Pc,
    state: &mut PartyState,
    config: &PartyConfig,
    difficulty: u8,
) -> Outcome {
    let Some(location) = state.resolve(pc.db_id, pc.object_id) else {
        return byte_reply(181, 2);
    };
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
    let mut out = Outcome::default();
    if reason == 0 {
        party.difficulty = difficulty;
        out.party_effects.push(PartyEffect::DifficultyChanged {
            location,
            difficulty,
        });
        out.party_effects.push(PartyEffect::Boundary(
            "❓ difficulty resize/sync 0x14021EDE0",
        ));
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
    out.party_effects
        .push(PartyEffect::Boundary("❓ PC metadata update 0x1405FAE30"));
    let team = state.teams[location.team].as_ref().unwrap();
    if team.mode != 0 {
        let current = team.parties[location.party].as_ref().unwrap().difficulty;
        let recipients = state.team_recipients(team);
        fanout(
            &mut out,
            recipients,
            Message {
                command: 275,
                body: vec![location.party as u8, current],
            },
        );
    }
    out.status = Status::Unsupported("❓ difficulty resize/metadata/report/downstream effects");
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
