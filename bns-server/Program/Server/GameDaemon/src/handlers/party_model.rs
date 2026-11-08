//! Clean-room party state for doc 14, build 381.8867.
//! Vectors retain native indexes. Do not compact vacant slots.
use super::*;

#[derive(Clone, Debug, Default)]
pub struct PartyConfig {
    /// Config +237. `World::arena` represents +237 AND +248.
    pub arena_enabled: bool,
    pub arena_task_mode: u8,
    pub default_difficulty: u8,
    pub suggestion_task: bool,
    /// Config +2736 byte +732: team-aware suggestion routing.
    pub team_suggestion_routing: bool,
    /// Config +2736 byte +537: optional party reports.
    pub trace_report: bool,
    /// Daemon +2728 pointer and pointee +32 enable rejection game-log output.
    pub rejection_aux_report: bool,
    pub vote_enabled: bool,
    pub maximum_grade: i32,
    pub rule_three_mode: i16,
    pub maximum_bid: i64,
    /// Table slot 711, record +24. Missing entries have maximum zero.
    pub difficulty_maximum: BTreeMap<u8, i8>,
}

#[derive(Clone, Debug)]
pub struct PartyPeer {
    pub pc: Pc,
    pub rights: u16,
}
#[derive(Clone, Debug, Default)]
pub struct PartyState {
    pub world_mask: u64,
    pub world_shift: u32,
    pub world_slots: Vec<Option<PartyPeer>>,
    /// Native uses CRT wcsicmp. ASCII keys use the exact portable subset.
    /// Non-ASCII comparisons require the Services locale adapter.
    pub names: BTreeMap<Vec<u16>, usize>,
    pub pc_teams: Vec<u64>,
    pub team_mask: u64,
    pub team_shift: u32,
    pub teams: Vec<Option<Team>>,
    /// None: unknown. A missing owner in a known map has an empty card box.
    pub invitation_boxes: Option<BTreeMap<u64, Vec<PartyInvitation>>>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartyInvitation {
    pub inviter_db: u64,
    pub inviter_runtime: u64,
    pub kind: u32,
}
#[derive(Clone, Debug, Default)]
pub struct Team {
    /// Team +40.
    pub runtime_id: u64,
    pub parties: Vec<Option<Party>>,
    pub indexed_parties: Vec<i32>,
    pub membership: BTreeMap<u64, i32>,
    pub leader_db_id: u64,
    pub leader_runtime_id: u64,
    pub invite_authority: u32,
    pub mode: u8,
}
#[derive(Clone, Debug, Default)]
pub struct Party {
    /// Party +48.
    pub runtime_id: u64,
    pub members: Vec<PartyMember>,
    pub difficulty: u8,
    pub invite_authority: u32,
    pub pickup_rule: i32,
    pub designated_id: u64,
    pub grade: u8,
    pub money: u64,
    pub difficulty_blockers: [i32; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyMember {
    pub slot: u8,
    pub db_id: u64,
    pub runtime_id: u64,
    pub state: u32,
    pub voted_for: u8,
    pub vote: u8,
}
impl Default for PartyMember {
    fn default() -> Self {
        Self {
            slot: 0,
            db_id: 0,
            runtime_id: 0,
            state: 0,
            voted_for: 0xff,
            vote: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartyLocation {
    pub team: usize,
    pub party: usize,
    pub member: usize,
}
fn high_word(id: u64) -> u16 {
    (id >> 16) as u16
}
fn index(id: u64, mask: u64, shift: u32) -> Option<usize> {
    usize::try_from((id & mask).checked_shr(shift)?).ok()
}
pub(super) fn name_prefix(name: &[u16]) -> &[u16] {
    &name[..name
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(name.len())]
}
impl PartyState {
    /// 0x1403F7D20. The native array pointer has no full-ID recheck.
    pub fn by_id(&self, id: u64) -> Option<&PartyPeer> {
        if id == 0 || high_word(id) != 1 {
            return None;
        }
        let i = index(id, self.world_mask, self.world_shift)?;
        if i >= 20000 {
            return None;
        }
        self.eligible(i)
    }
    fn eligible(&self, i: usize) -> Option<&PartyPeer> {
        self.world_slots
            .get(i)?
            .as_ref()
            .filter(|peer| peer.rights & 6 != 0)
    }
    /// 0x1403F7EF0. The caller has already consumed every counted wire unit.
    pub fn by_name(&self, name: &[u16]) -> Option<&PartyPeer> {
        let key = name_prefix(name);
        if key.is_empty() {
            return None;
        }
        let PartyNameLookup::Slot(slot) = self.name_slot(key) else {
            return None;
        };
        self.eligible(slot)
    }
    pub fn name_slot(&self, name: &[u16]) -> PartyNameLookup {
        let key = name_prefix(name);
        if key.is_empty() {
            return PartyNameLookup::Missing;
        }
        if key.iter().any(|c| *c > 127)
            || self
                .names
                .keys()
                .any(|k| name_prefix(k).iter().any(|c| *c > 127))
        {
            return PartyNameLookup::Unknown;
        }
        let fold = |c: &u16| if (65..=90).contains(c) { *c + 32 } else { *c };
        self.names
            .iter()
            .find_map(|(name, slot)| {
                name_prefix(name)
                    .iter()
                    .map(fold)
                    .eq(key.iter().map(fold))
                    .then_some(PartyNameLookup::Slot(*slot))
            })
            .unwrap_or(PartyNameLookup::Missing)
    }
    pub(super) fn by_name_slot(&self, slot: usize) -> Option<&PartyPeer> {
        self.eligible(slot)
    }
    pub fn target(&self, id: u64, name: &[u16]) -> Option<&PartyPeer> {
        if id != 0 {
            self.by_id(id)
        } else {
            self.by_name(name)
        }
    }
    /// 0x1401D3F70 / 0x1401EE320, mode 3. Access requires exclusive adapter ownership.
    pub fn resolve(&self, db: u64, runtime: u64) -> Option<PartyLocation> {
        if db == 0 || runtime == 0 || high_word(runtime) != 1 {
            return None;
        }
        let team_id = *self.pc_teams.get((runtime & 0xffff) as usize)?;
        if high_word(team_id) != 64 {
            return None;
        }
        let ti = index(team_id, self.team_mask, self.team_shift)?;
        let team = self.teams.get(ti)?.as_ref()?;
        let pi = usize::try_from(*team.membership.get(&db)?).ok()?;
        let party = team.parties.get(pi)?.as_ref()?;
        let mi = party
            .members
            .iter()
            .position(|m| m.db_id == db && m.runtime_id == runtime && m.state == 3)?;
        Some(PartyLocation {
            team: ti,
            party: pi,
            member: mi,
        })
    }
    pub(super) fn recipients(&self, party: &Party) -> Vec<u64> {
        party
            .members
            .iter()
            .filter(|m| m.state == 3)
            .filter_map(|m| self.by_id(m.runtime_id).map(|p| p.pc.object_id))
            .collect()
    }
    pub(super) fn team_recipients(&self, team: &Team) -> Vec<u64> {
        team.indexed_parties
            .iter()
            .filter_map(|i| usize::try_from(*i).ok())
            .filter_map(|i| team.parties.get(i)?.as_ref())
            .flat_map(|p| self.recipients(p))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyTask {
    pub command: u16,
    pub caster: u64,
    pub zone: u64,
    pub kind: PartyTaskKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartyTaskKind {
    LeaveParty,
    LeaveTeam {
        mode: u8,
    },
    DelegateParty {
        target: u64,
    },
    BanishParty {
        slot: u8,
    },
    Pickup {
        rule: u8,
        grade: u8,
        target: u64,
        money: u64,
    },
    Difficulty {
        difficulty: u8,
    },
    PartyAuthority {
        authority: u8,
    },
    TeamAuthority {
        authority: u8,
    },
    InviteParty {
        inviter_db: u64,
        inviter: u64,
        correlation: u32,
        options: [u8; 6],
    },
    JoinParty {
        invitation: u64,
        correlation: u32,
        difficulty: u8,
    },
    Reject {
        group: u8,
        invitation: u64,
        correlation: u8,
        kind: u8,
    },
    BanishTeam {
        party: u16,
        slot: u8,
    },
    MoveMember {
        source: (u16, u8, u64),
        destination: (u16, u8, u64),
    },
    DelegateTeam {
        target: u64,
    },
    NominateLeader {
        target: u64,
    },
    TeamDifficulty {
        selector: u8,
        difficulty: u8,
    },
    SuggestParty {
        requester: u64,
        options: [u8; 6],
    },
    InviteTeam {
        mode: u8,
        inviter_db: u64,
        inviter: u64,
        options: [u8; 5],
        target: u64,
    },
    JoinTeam {
        mode: u8,
        invitation: u64,
        other: u64,
    },
    Vote {
        target: u8,
        vote: u8,
    },
    ClearMembers,
    /// Native kind 207, vtable 0x140B1DDD8, qwords +144/+152.
    UpdatePcInfo {
        team: u64,
        party: u64,
    },
    Auction {
        operation: AuctionOperation,
        value: u64,
        byte: u8,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuctionOperation {
    Pass,
    Bid,
    Finish,
    Distribute,
    GiveUp,
}

/// Ordered effects supplement the existing reply/broadcast lists.
/// An adapter must apply them in order, including sends before vote clearing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartyEffect {
    Send {
        to: u64,
        message: Message,
    },
    VoteCleared {
        location: PartyLocation,
    },
    TeamAuthorityChanged {
        team: usize,
        authority: u8,
    },
    PartyAuthorityChanged {
        location: PartyLocation,
        authority: u8,
    },
    PickupChanged {
        location: PartyLocation,
    },
    DifficultyChanged {
        location: PartyLocation,
        difficulty: u8,
    },
    PcInfoChanged {
        runtime: u64,
        team_mode: u8,
        difficulty: u8,
    },
    InvitationRemoved {
        owner: u64,
        invitation: PartyInvitation,
    },
    AssertionReport,
    /// ❓ This operation is not read. It is NOT a completed backend effect.
    Boundary(&'static str),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyRegistration {
    pub value: u64,
    pub route: u32,
}

/// CRT locale lookup result. An absent locale adapter is not a missing PC.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PartyNameLookup {
    #[default]
    Unknown,
    Missing,
    Slot(usize),
}

impl Team {
    /// 0x1401F31D0. Party index is a signed native byte.
    pub fn allows_invite(&self, mode: u8, party: u8, db: u64, runtime: u64) -> bool {
        if self.leader_db_id == db && self.leader_runtime_id == runtime {
            return true;
        }
        if self.invite_authority == 0 {
            return false;
        }
        let leader = (party as i8 >= 0)
            .then(|| self.parties.get(usize::from(party)))
            .flatten()
            .and_then(|p| p.as_ref())
            .and_then(|p| p.members.first())
            .is_some_and(|m| m.db_id == db && m.runtime_id == runtime);
        match self.invite_authority {
            1 => leader && matches!(mode, 2 | 3),
            2 => {
                if leader {
                    matches!(mode, 2 | 3)
                } else {
                    mode == 2
                }
            }
            _ => false,
        }
    }
}

/// Result of an unread leader resolver. Missing differs from an absent adapter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PartyLeaderLookup {
    #[default]
    Unknown,
    Missing,
    Runtime(u64),
}

/// Opaque game managers only. Excluded restriction queries have no adapter here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartyOperation {
    LeaveTeam,
    LeaveParty,
    DelegateParty {
        location: PartyLocation,
        target_db: u64,
        target_runtime: u64,
    },
    TransferTeamLeader {
        team: usize,
        target_db: u64,
        target_runtime: u64,
    },
    BanishParty {
        slot: u8,
    },
    BanishTeam {
        party: u16,
        slot: u8,
    },
    MoveMember {
        source: (u16, u8, u64),
        destination: (u16, u8, u64),
    },
    DelegateTeam {
        target_db: u64,
        target_runtime: u64,
    },
    NominateLeader {
        target_db: u64,
        target_runtime: u64,
    },
    ClearMembers,
}
