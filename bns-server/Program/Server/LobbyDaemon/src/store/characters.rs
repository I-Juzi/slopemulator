//! Local lifecycle persistence. Keep internal states outside wire-ready rows.
use super::*;
use crate::character::{
    procedure_error, CreateCharacter, CreatedCharacter, InternalState as State,
};

impl MemoryStore {
    pub(super) fn character_failure(&mut self, op: &'static str) -> io::Result<()> {
        match self.character_failures.remove(op) {
            Some(code) => Err(procedure_error(code)),
            None => Ok(()),
        }
    }
    fn character_mut(&mut self, pcid: u64) -> io::Result<&mut Character> {
        self.characters
            .values_mut()
            .flatten()
            .find(|c| c.pcid == pcid)
            .ok_or_else(absent)
    }
    pub(super) fn transition(
        &mut self,
        op: &'static str,
        pcid: u64,
        from: State,
        to: State,
    ) -> io::Result<()> {
        self.character_failure(op)?;
        if self.character_states.get(&pcid) != Some(&from) {
            return Err(procedure_error(1));
        }
        self.character_mut(pcid)?.state = to.wire();
        self.character_states.insert(pcid, to);
        Ok(())
    }
    pub(super) fn check_name(&mut self, name: &[u16]) -> io::Result<()> {
        self.character_failure("check_name")?;
        if name.is_empty() || name.len() > 32 || name.contains(&0) {
            return Err(procedure_error(1));
        }
        let pure = |n: &[u16]| {
            String::from_utf16_lossy(n)
                .replace(' ', "")
                .to_ascii_lowercase()
        };
        let requested = pure(name);
        if self
            .characters
            .values()
            .flatten()
            .any(|c| pure(&c.name) == requested)
        {
            return Err(procedure_error(2));
        }
        if self.limited_names.iter().any(|n| pure(n) == requested) {
            return Err(procedure_error(3));
        }
        Ok(())
    }
    pub(super) fn create(&mut self, request: CreateCharacter) -> io::Result<u64> {
        self.character_failure("try_create")?;
        if !self.accounts.contains_key(&request.account)
            || request.slot == [0; 16]
            || request.world <= 0
            || request.right != 1
        {
            return Err(procedure_error(1));
        }
        self.check_name(&request.name)?;
        if self
            .characters
            .values()
            .flatten()
            .any(|c| c.slot == request.slot)
        {
            return Err(procedure_error(1));
        }
        let pcid = self
            .characters
            .values()
            .flatten()
            .map(|c| c.pcid)
            .chain(self.character_states.keys().copied())
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .filter(|n| *n <= i64::MAX as u64)
            .ok_or_else(|| procedure_error(1))?;
        self.characters
            .entry(request.account)
            .or_default()
            .push(Character {
                pcid,
                slot: request.slot,
                world: request.world,
                name: request.name,
                showcase: vec![],
                deletion_time: 0,
                state: 0,
                jumping_state: 0,
                last_play_end: 0,
            });
        self.character_states.insert(pcid, State::Creating);
        Ok(pcid)
    }
    pub(super) fn finish_create(&mut self, pcid: u64, created: CreatedCharacter) -> io::Result<()> {
        check_blob(&created.showcase)?;
        if created.job <= 0 || created.level <= 0 || created.showcase.is_empty() {
            return Err(procedure_error(1));
        }
        self.transition("finish_create", pcid, State::Creating, State::Normal)?;
        self.character_mut(pcid)?.showcase = created.showcase;
        Ok(())
    }
    pub(super) fn cancel_create(&mut self, pcid: u64) -> io::Result<()> {
        self.transition("cancel_create", pcid, State::Creating, State::Deleted)?;
        for rows in self.characters.values_mut() {
            rows.retain(|c| c.pcid != pcid);
        }
        // Keep a tombstone so the local ID source cannot silently reuse this PCID.
        Ok(())
    }
    pub(super) fn finish_delete(&mut self, pcid: u64, seconds: u32) -> io::Result<()> {
        if seconds > i32::MAX as u32 {
            return Err(procedure_error(1));
        }
        let state = if seconds == 0 {
            State::Deleted
        } else {
            State::Waiting
        };
        self.transition("finish_delete", pcid, State::Deleting, state)?;
        self.character_mut(pcid)?.last_play_end = seconds;
        if seconds == 0 {
            for rows in self.characters.values_mut() {
                rows.retain(|c| c.pcid != pcid);
            }
        }
        Ok(())
    }
}
