//! Database interface (spec README.md). The 348 daemon calls stored
//! procedures through its DB layer; this trait names one method per call.
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreError;

/// Rows of one Pc: registers (id → value), completed (id → (step, time)),
/// rewards (id → reward step).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PcRows {
    pub registers: BTreeMap<u16, i64>,
    pub completed: BTreeMap<u16, (u16, i64)>,
    pub rewards: BTreeMap<u16, u16>,
}

pub trait Store: Send + Sync + 'static {
    /// LoadRegisters + LoadCompleted.
    fn load(&self, pcdbid: i64) -> Result<PcRows, StoreError>;
    fn insert_registers(&self, pcdbid: i64, rows: &[(u16, i64)]) -> Result<(), StoreError>;
    fn update_registers(&self, pcdbid: i64, rows: &[(u16, i64)]) -> Result<(), StoreError>;
    fn insert_completed(&self, pcdbid: i64, rows: &[(u16, u16, i64)]) -> Result<(), StoreError>;
    fn update_completed(&self, pcdbid: i64, rows: &[(u16, u16, i64)]) -> Result<(), StoreError>;
    fn take_step_reward(&self, pcdbid: i64, rows: &[(u16, u16)]) -> Result<(), StoreError>;
    fn delete_completed(&self, pcdbid: i64, id: u16) -> Result<(), StoreError>;
    /// DeleteCompletedAll + DeleteRegistersAll.
    fn delete_all(&self, pcdbid: i64) -> Result<(), StoreError>;
}

/// In-memory store for tests and the stand-alone binary.
#[derive(Default)]
pub struct MemoryStore {
    pub pcs: Mutex<HashMap<i64, PcRows>>,
    /// When set, every call fails (tests of the keep-modified path).
    pub fail: std::sync::atomic::AtomicBool,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
    fn with<T>(&self, pcdbid: i64, f: impl FnOnce(&mut PcRows) -> T) -> Result<T, StoreError> {
        if self.fail.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(StoreError);
        }
        let mut pcs = self.pcs.lock().unwrap_or_else(|e| e.into_inner());
        Ok(f(pcs.entry(pcdbid).or_default()))
    }
    pub fn rows(&self, pcdbid: i64) -> PcRows {
        let pcs = self.pcs.lock().unwrap_or_else(|e| e.into_inner());
        pcs.get(&pcdbid).cloned().unwrap_or_default()
    }
}

impl Store for MemoryStore {
    fn load(&self, pcdbid: i64) -> Result<PcRows, StoreError> {
        self.with(pcdbid, |r| r.clone())
    }
    fn insert_registers(&self, pcdbid: i64, rows: &[(u16, i64)]) -> Result<(), StoreError> {
        self.with(pcdbid, |r| r.registers.extend(rows.iter().copied()))
    }
    fn update_registers(&self, pcdbid: i64, rows: &[(u16, i64)]) -> Result<(), StoreError> {
        self.insert_registers(pcdbid, rows)
    }
    fn insert_completed(&self, pcdbid: i64, rows: &[(u16, u16, i64)]) -> Result<(), StoreError> {
        self.with(pcdbid, |r| {
            r.completed
                .extend(rows.iter().map(|&(id, step, time)| (id, (step, time))))
        })
    }
    fn update_completed(&self, pcdbid: i64, rows: &[(u16, u16, i64)]) -> Result<(), StoreError> {
        self.insert_completed(pcdbid, rows)
    }
    fn take_step_reward(&self, pcdbid: i64, rows: &[(u16, u16)]) -> Result<(), StoreError> {
        self.with(pcdbid, |r| r.rewards.extend(rows.iter().copied()))
    }
    fn delete_completed(&self, pcdbid: i64, id: u16) -> Result<(), StoreError> {
        self.with(pcdbid, |r| {
            r.completed.remove(&id);
            r.rewards.remove(&id);
        })
    }
    fn delete_all(&self, pcdbid: i64) -> Result<(), StoreError> {
        self.with(pcdbid, |r| *r = PcRows::default())
    }
}
