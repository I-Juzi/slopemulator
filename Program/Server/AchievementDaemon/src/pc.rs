//! Pc state and achievement logic (spec README.md): register operations,
//! checkComplete, availableTakeReward and the data packets.
use crate::protocol::{game_cmd, packet};
use crate::rules::{Achievement, Rules};
use crate::store::PcRows;
use bloodlib::packet::Value;
use std::collections::BTreeMap;

/// Modified marker of one row: 0 clean, 1 insert, 2 update.
pub const CLEAN: u8 = 0;
pub const INSERT: u8 = 1;
pub const UPDATE: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot<T> {
    pub value: T,
    pub modified: u8,
}

impl<T> Slot<T> {
    fn mark_update(&mut self) {
        if self.modified != INSERT {
            self.modified = UPDATE;
        }
    }
}

/// Register row metadata stays with the row, as in the native Pc::Register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterSlot {
    pub value: i64,
    pub version: u16,
    pub modified: u8,
    pub visible: bool,
}

impl RegisterSlot {
    fn mark_update(&mut self) {
        if self.modified != INSERT {
            self.modified = UPDATE;
        }
    }
}

/// Completed step and its time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Done {
    pub step: u16,
    pub time: i64,
}

/// Fields copied from the PcLogin event.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Profile {
    pub world_id: u16,
    pub level: u16,
    pub mastery_level: u16,
    pub faction: u8,
    pub faction2: u8,
    pub inventory: u8,
    pub depot: u8,
    pub wardrobe: u8,
    pub race: u8,
    pub sex: u8,
    pub job: u8,
    pub pcid: i64,
    pub duel_solo_grade: u8,
    pub duel_team_grade: u8,
    pub party_battle_grades: [u8; 3],
    pub name: Vec<u16>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pc {
    pub pcdbid: i64,
    pub tid: i64,
    pub hint: i64,
    pub cluster_id: i16,
    pub profile: Profile,
    pub registers: BTreeMap<u16, RegisterSlot>,
    pub completed: BTreeMap<u16, Slot<Done>>,
    pub rewards: BTreeMap<u16, Slot<u16>>,
    /// Sum of the selected completed records' score_sum values.
    pub score: i32,
    /// LoadRegisters emits these before SendAllData (0x140003D60).
    pub load_notifications: Vec<Vec<u8>>,
}

/// Rows with a modified marker, collected by updatePc.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Changes {
    pub insert_registers: Vec<(u16, u16, i64)>,
    pub update_registers: Vec<(u16, u16, i64)>,
    pub insert_completed: Vec<(u16, u16, i64)>,
    pub update_completed: Vec<(u16, u16, i64)>,
    pub rewards: Vec<(u16, u16)>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.insert_registers.is_empty()
            && self.update_registers.is_empty()
            && self.insert_completed.is_empty()
            && self.update_completed.is_empty()
            && self.rewards.is_empty()
    }
}

/// availableTakeReward result codes.
pub mod take {
    pub const OK: i16 = 0;
    pub const NOT_AVAILABLE: i16 = 5;
    pub const NOT_FOUND: i16 = 6;
    pub const DEPRECATED: i16 = 7;
    pub const ALREADY_TAKEN: i16 = 8;
    pub const NOT_COMPLETED: i16 = 9;
}

impl Pc {
    pub fn new(
        pcdbid: i64,
        tid: i64,
        cluster_id: i16,
        profile: Profile,
        rows: PcRows,
        rules: &Rules,
    ) -> Self {
        let mut pc = Pc {
            pcdbid,
            tid,
            cluster_id,
            profile,
            ..Default::default()
        };
        for (id, (version, value)) in rows.registers {
            let Some(register) = rules.register(id, pc.job()) else {
                continue;
            };
            let mismatch = version != register.version;
            if mismatch {
                pc.load_notifications.push(pc.register_reset_notify(
                    id,
                    version,
                    register.version,
                    value,
                ));
            }
            pc.registers.insert(
                id,
                RegisterSlot {
                    value: if mismatch { 0 } else { value },
                    version: register.version,
                    modified: if mismatch { UPDATE } else { CLEAN },
                    visible: register.visible(),
                },
            );
        }
        for (id, (step, time)) in rows.completed {
            pc.completed.insert(
                id,
                Slot {
                    value: Done { step, time },
                    modified: CLEAN,
                },
            );
        }
        for (id, step) in rows.rewards {
            pc.rewards.insert(
                id,
                Slot {
                    value: step,
                    modified: CLEAN,
                },
            );
        }
        pc.score = pc.compute_score(rules);
        pc
    }

    pub fn job(&self) -> u8 {
        self.profile.job
    }

    /// Use the selected completed record's derived score_sum (0x140016CA0).
    pub fn compute_score(&self, rules: &Rules) -> i32 {
        self.completed
            .iter()
            .filter_map(|(&id, done)| rules.record(id, done.value.step, self.job()))
            .fold(0i32, |score, r| score.wrapping_add(r.score_sum))
    }

    pub fn register(&self, id: u16) -> i64 {
        self.registers.get(&id).map_or(0, |s| s.value)
    }

    pub fn completed_step(&self, id: u16) -> u16 {
        self.completed.get(&id).map_or(0, |s| s.value.step)
    }

    /// Write a register value and mark it. New rows are marked only when the
    /// register is persistant or progress-show.
    fn store(&mut self, rules: &Rules, id: u16, value: i64) {
        let tracked = rules
            .register(id, self.job())
            .is_some_and(|r| r.persistant || r.progress_show);
        match self.registers.get_mut(&id) {
            Some(slot) => {
                if slot.value != value {
                    slot.value = value;
                    if tracked {
                        slot.mark_update();
                    }
                }
            }
            None => {
                let modified = if tracked { INSERT } else { CLEAN };
                let register = rules.register(id, self.job()).expect("validated register");
                self.registers.insert(
                    id,
                    RegisterSlot {
                        value,
                        modified,
                        version: register.version,
                        visible: register.visible(),
                    },
                );
            }
        }
    }

    fn after_change(&mut self, rules: &Rules, id: u16, now: i64, out: &mut Vec<Vec<u8>>) {
        for key in rules.linked(id, self.job()) {
            if let Some(record) = rules.achievements.get(key) {
                self.check_complete(rules, record, now, out);
            }
        }
    }

    /// AddRegister: job filter, clamp to max-value (when not 0) and to >= 0.
    pub fn add_register(
        &mut self,
        rules: &Rules,
        id: u16,
        delta: i64,
        now: i64,
        out: &mut Vec<Vec<u8>>,
    ) -> bool {
        // Job filter: only the row of the Pc job or of job-none applies.
        let Some(register) = rules.register(id, self.job()) else {
            return false;
        };
        let mut value = self.register(id).saturating_add(delta);
        if register.max_value != 0 && value > register.max_value {
            value = register.max_value;
        }
        value = value.max(0);
        self.store(rules, id, value);
        self.after_change(rules, id, now, out);
        true
    }

    /// SetRegister: no clamp.
    pub fn set_register(
        &mut self,
        rules: &Rules,
        id: u16,
        value: i64,
        now: i64,
        out: &mut Vec<Vec<u8>>,
    ) -> bool {
        if rules.register(id, self.job()).is_none() {
            return false;
        }
        self.store(rules, id, value);
        self.after_change(rules, id, now, out);
        true
    }

    /// BitwiseOrRegister: no clamp.
    pub fn or_register(
        &mut self,
        rules: &Rules,
        id: u16,
        bits: i64,
        now: i64,
        out: &mut Vec<Vec<u8>>,
    ) -> bool {
        let value = self.register(id) | bits;
        self.set_register(rules, id, value, now, out)
    }

    fn conditions_hold(&self, record: &Achievement) -> bool {
        record.complete_from_client
            || record
                .conditions
                .iter()
                .all(|c| c.op.holds(self.register(c.register), c.value))
    }

    /// Check one exact record, not a next-step loop (0x140017DD0).
    pub fn check_complete(
        &mut self,
        rules: &Rules,
        record: &Achievement,
        now: i64,
        out: &mut Vec<Vec<u8>>,
    ) {
        let id = record.id;
        let current = self.completed_step(id);
        if record.deprecated
            || (self.completed.contains_key(&id) && record.step <= current)
            || !self.conditions_hold(record)
        {
            return;
        }
        let done = Done {
            step: record.step,
            time: now,
        };
        match self.completed.get_mut(&id) {
            Some(slot) => {
                slot.value = done;
                slot.mark_update();
            }
            None => {
                self.completed.insert(
                    id,
                    Slot {
                        value: done,
                        modified: INSERT,
                    },
                );
            }
        }
        self.score = self.compute_score(rules);
        // SetRegister must run linked completion, even when zero is unchanged.
        // Store the completed step first to prevent recursive re-completion.
        for condition in &record.conditions {
            if condition.clear {
                self.set_register(rules, condition.register, 0, now, out);
            }
        }
        let last = !rules
            .achievements
            .contains_key(&(id, record.step.wrapping_add(1), record.job));
        out.push(self.completed_notify(id, current, record.step, last));
    }

    /// availableTakeReward for an existing record of (id, step).
    pub fn available_take_reward(&self, rules: &Rules, record: &Achievement, job: u8) -> i16 {
        if self.completed_step(record.id) == 0 && !self.rewards.contains_key(&record.id) {
            return take::NOT_AVAILABLE;
        }
        if record.deprecated {
            return take::DEPRECATED;
        }
        if record.complete_from_client || record.conditions.is_empty() {
            return take::NOT_AVAILABLE;
        }
        if self.completed_step(record.id) < record.step {
            return take::NOT_COMPLETED;
        }
        if self
            .rewards
            .get(&record.id)
            .is_some_and(|r| r.value >= record.step)
        {
            return take::ALREADY_TAKEN;
        }
        let taken = self.rewards.get(&record.id).map_or(0, |r| r.value);
        // Native scans backward, so a nearer reward takes priority over a gap.
        for step in ((taken.saturating_add(1))..record.step).rev() {
            let Some(previous) = rules.record(record.id, step, job) else {
                return take::NOT_AVAILABLE;
            };
            if previous.has_reward {
                return take::NOT_COMPLETED;
            }
        }
        take::OK
    }

    /// CompleteTakeReward: check, then store the reward step (0x140017B80).
    pub fn take_reward(&mut self, rules: &Rules, record: &Achievement, job: u8) -> i16 {
        let reason = self.available_take_reward(rules, record, job);
        if reason == take::OK {
            match self.rewards.get_mut(&record.id) {
                Some(slot) => {
                    slot.value = record.step;
                    slot.modified = UPDATE;
                }
                None => {
                    self.rewards.insert(
                        record.id,
                        Slot {
                            value: record.step,
                            modified: UPDATE,
                        },
                    );
                }
            }
        }
        reason
    }

    /// Rows with a marker (updatePc input).
    pub fn changes(&self) -> Changes {
        let mut c = Changes::default();
        for (&id, s) in &self.registers {
            match s.modified {
                INSERT => c.insert_registers.push((id, s.version, s.value)),
                UPDATE => c.update_registers.push((id, s.version, s.value)),
                _ => {}
            }
        }
        for (&id, s) in &self.completed {
            match s.modified {
                INSERT => c.insert_completed.push((id, s.value.step, s.value.time)),
                UPDATE => c.update_completed.push((id, s.value.step, s.value.time)),
                _ => {}
            }
        }
        for (&id, s) in &self.rewards {
            if s.modified != CLEAN {
                c.rewards.push((id, s.value));
            }
        }
        c
    }

    pub fn clear_registers(&mut self, marker: u8) {
        self.registers
            .values_mut()
            .filter(|s| s.modified == marker)
            .for_each(|s| s.modified = CLEAN);
    }
    pub fn clear_completed(&mut self, marker: u8) {
        self.completed
            .values_mut()
            .filter(|s| s.modified == marker)
            .for_each(|s| s.modified = CLEAN);
    }
    pub fn clear_rewards(&mut self) {
        self.rewards.values_mut().for_each(|s| s.modified = CLEAN);
    }

    /// Data packet (cmd 7) as fragments. `changed` selects SendChangedData
    /// (only marked rows) instead of SendAllData.
    pub fn data_packets(&self, changed: bool) -> Vec<Vec<u8>> {
        let keep = |m: u8| !changed || m != CLEAN;
        let registers: Vec<_> = self
            .registers
            .iter()
            .filter(|(_, s)| s.visible && keep(s.modified))
            .map(|(&id, s)| vec![Value::I16(id as i16), Value::I64(s.value)])
            .collect();
        let completed: Vec<_> = self
            .completed
            .iter()
            .filter(|(_, s)| keep(s.modified))
            .map(|(&id, s)| {
                vec![
                    Value::I16(id as i16),
                    Value::I16(s.value.step as i16),
                    Value::I64(s.value.time),
                ]
            })
            .collect();
        let rewards: Vec<_> = self
            .rewards
            .iter()
            .filter(|(_, s)| keep(s.modified))
            .map(|(&id, s)| vec![Value::I16(id as i16), Value::I16(s.value as i16)])
            .collect();
        if changed && registers.is_empty() && completed.is_empty() && rewards.is_empty() {
            return Vec::new();
        }
        packet::fragments::compose_fragments(
            game_cmd::DATA,
            "hiqqqihcChq;Chhq;Chh;",
            &[
                Value::I64(self.tid),
                Value::I64(self.pcdbid),
                Value::I64(self.hint),
                Value::I32(self.score),
                Value::I16(self.cluster_id),
                Value::U8(changed as u8),
                Value::Repeat(registers),
                Value::Repeat(completed),
                Value::Repeat(rewards),
            ],
        )
        .unwrap_or_default()
    }

    /// RegisterResetNotify (cmd 12), native LoadRegisters at 0x140003D60.
    pub fn register_reset_notify(&self, id: u16, old: u16, new: u16, value: i64) -> Vec<u8> {
        packet(
            "hsqqqhhhqq",
            &[
                Value::I16(game_cmd::REGISTER_RESET_NOTIFY),
                Value::String(self.profile.name.clone()),
                Value::I64(self.pcdbid),
                Value::I64(self.profile.pcid),
                Value::I64(self.tid),
                Value::I16(id as i16),
                Value::I16(old as i16),
                Value::I16(new as i16),
                Value::I64(value),
                Value::I64(0),
            ],
        )
    }

    /// CompletedNotify (cmd 11): pcname, pcdbid, pcid, tid, id, old step, new step, isLastStep.
    pub fn completed_notify(&self, id: u16, old: u16, new: u16, last: bool) -> Vec<u8> {
        packet(
            "hsqqqhhhc",
            &[
                Value::I16(game_cmd::COMPLETED_NOTIFY),
                Value::String(self.profile.name.clone()),
                Value::I64(self.pcdbid),
                Value::I64(self.profile.pcid),
                Value::I64(self.tid),
                Value::I16(id as i16),
                Value::I16(old as i16),
                Value::I16(new as i16),
                Value::U8(last as u8),
            ],
        )
    }

    /// Take-reward reply: cmd 9 `hqqhh` or cmd 10 `hEqqhh`
    /// [reason], transaction id, pcdbid, achievement id, reward step (0x1400183E0).
    pub fn take_reward_reply(&self, transaction: i64, id: u16, step: u16, reason: i16) -> Vec<u8> {
        if reason == take::OK {
            packet(
                "hqqhh",
                &[
                    Value::I16(game_cmd::TAKE_REWARD_OK),
                    Value::I64(transaction),
                    Value::I64(self.pcdbid),
                    Value::I16(id as i16),
                    Value::I16(step as i16),
                ],
            )
        } else {
            packet(
                "hEqqhh",
                &[
                    Value::I16(game_cmd::TAKE_REWARD_FAIL),
                    Value::I16(reason),
                    Value::I64(transaction),
                    Value::I64(self.pcdbid),
                    Value::I16(id as i16),
                    Value::I16(step as i16),
                ],
            )
        }
    }
}
