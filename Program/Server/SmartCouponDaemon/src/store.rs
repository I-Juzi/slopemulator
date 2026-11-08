//! Store interface for the SmartCoupon procedures (spec 04-database.md).
//! The procedure bodies are not in the available database dumps, so
//! `MemoryStore` models only the visible contract.
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

pub type Account = [u8; 16];

/// GameEventInfo eventType and GameEventLogEvent target type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Account = 1,
    Character = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventLog {
    pub log_id: i64,
    pub target: Target,
    pub account: Account,
    /// Zero for account events (bound as NULL).
    pub world: i16,
    /// Zero for account events (bound as NULL).
    pub pcdbid: i64,
    pub event_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Publication {
    pub publication_id: i64,
    pub account: Account,
    pub coupon_event_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreError;

/// Lists returned by the procedures hold at most this many rows.
pub const MAX_LIST: usize = 128;

pub trait Store: Send + Sync + 'static {
    /// usp_RegisterGameEventLog. Returns the new log id.
    fn register_game_event_log(
        &self,
        target: Target,
        account: &Account,
        world: i16,
        pcdbid: i64,
        event_id: i32,
    ) -> Result<i64, StoreError>;
    /// usp_ProcessAccountGameEventLog or usp_ProcessCharacterGameEventLog.
    fn process_game_event_log(&self, log: &EventLog) -> Result<Vec<Publication>, StoreError>;
    /// usp_UpdateStatusToProcessFailure.
    fn update_status_to_process_failure(&self, log: &EventLog) -> Result<(), StoreError>;
    /// usp_UpdateStatusToPublicationSuccess.
    fn update_status_to_publication_success(
        &self,
        publication: &Publication,
        accept_tid: i64,
    ) -> Result<(), StoreError>;
    /// usp_UpdateStatusToPublicationFailure.
    fn update_status_to_publication_failure(
        &self,
        publication: &Publication,
    ) -> Result<(), StoreError>;
    /// usp_LoadGameEventLogList (start-up).
    fn load_game_event_log_list(&self) -> Result<Vec<EventLog>, StoreError>;
    /// usp_LoadCouponPublicationList (start-up).
    fn load_coupon_publication_list(&self) -> Result<Vec<Publication>, StoreError>;
    /// usp_LoadSingleCouponPublication. Zero means no coupon event.
    fn load_single_coupon_publication(
        &self,
        publication_id: i64,
        account: &Account,
    ) -> Result<i32, StoreError>;
    /// usp_LoadRetryReadyGameEventLogList.
    fn load_retry_ready_game_event_log_list(&self) -> Result<Vec<EventLog>, StoreError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogStatus {
    Registered,
    Processed,
    ProcessFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicationStatus {
    Ready,
    Success { accept_tid: i64 },
    Failure,
}

#[derive(Default)]
struct Inner {
    next_log: i64,
    next_publication: i64,
    logs: BTreeMap<i64, (EventLog, LogStatus)>,
    publications: BTreeMap<i64, (Publication, PublicationStatus)>,
    /// Local rule: game event id -> coupon event ids to publish.
    rules: HashMap<i32, Vec<i32>>,
    failing: bool,
}

/// In-memory store. Coupon rules are a local stand-in for the SQL logic.
#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<Inner>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_rule(&self, event_id: i32, coupon_event_id: i32) {
        self.lock()
            .rules
            .entry(event_id)
            .or_default()
            .push(coupon_event_id);
    }
    /// Make every later call fail (test hook).
    pub fn set_failing(&self, failing: bool) {
        self.lock().failing = failing;
    }
    pub fn log_status(&self, log_id: i64) -> Option<LogStatus> {
        self.lock().logs.get(&log_id).map(|(_, s)| *s)
    }
    pub fn publication_status(&self, publication_id: i64) -> Option<PublicationStatus> {
        self.lock()
            .publications
            .get(&publication_id)
            .map(|(_, s)| *s)
    }
    pub fn set_log_status(&self, log_id: i64, status: LogStatus) {
        if let Some(entry) = self.lock().logs.get_mut(&log_id) {
            entry.1 = status;
        }
    }
    pub fn insert_publication(&self, account: Account, coupon_event_id: i32) -> i64 {
        let mut inner = self.lock();
        inner.next_publication += 1;
        let publication_id = inner.next_publication;
        inner.publications.insert(
            publication_id,
            (
                Publication {
                    publication_id,
                    account,
                    coupon_event_id,
                },
                PublicationStatus::Ready,
            ),
        );
        publication_id
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
    fn checked(&self) -> Result<std::sync::MutexGuard<'_, Inner>, StoreError> {
        let inner = self.lock();
        if inner.failing {
            Err(StoreError)
        } else {
            Ok(inner)
        }
    }
}

impl Store for MemoryStore {
    fn register_game_event_log(
        &self,
        target: Target,
        account: &Account,
        world: i16,
        pcdbid: i64,
        event_id: i32,
    ) -> Result<i64, StoreError> {
        let mut inner = self.checked()?;
        inner.next_log += 1;
        let log_id = inner.next_log;
        let log = EventLog {
            log_id,
            target,
            account: *account,
            world,
            pcdbid,
            event_id,
        };
        inner.logs.insert(log_id, (log, LogStatus::Registered));
        Ok(log_id)
    }
    fn process_game_event_log(&self, log: &EventLog) -> Result<Vec<Publication>, StoreError> {
        let mut inner = self.checked()?;
        match inner.logs.get(&log.log_id) {
            Some((_, LogStatus::Registered | LogStatus::ProcessFailure)) => (),
            _ => return Err(StoreError),
        }
        let coupons = inner.rules.get(&log.event_id).cloned().unwrap_or_default();
        let mut out = Vec::new();
        for coupon_event_id in coupons.into_iter().take(MAX_LIST) {
            inner.next_publication += 1;
            let publication = Publication {
                publication_id: inner.next_publication,
                account: log.account,
                coupon_event_id,
            };
            inner.publications.insert(
                publication.publication_id,
                (publication, PublicationStatus::Ready),
            );
            out.push(publication);
        }
        if let Some(entry) = inner.logs.get_mut(&log.log_id) {
            entry.1 = LogStatus::Processed;
        }
        Ok(out)
    }
    fn update_status_to_process_failure(&self, log: &EventLog) -> Result<(), StoreError> {
        let mut inner = self.checked()?;
        let entry = inner.logs.get_mut(&log.log_id).ok_or(StoreError)?;
        entry.1 = LogStatus::ProcessFailure;
        Ok(())
    }
    fn update_status_to_publication_success(
        &self,
        publication: &Publication,
        accept_tid: i64,
    ) -> Result<(), StoreError> {
        let mut inner = self.checked()?;
        let entry = inner
            .publications
            .get_mut(&publication.publication_id)
            .ok_or(StoreError)?;
        entry.1 = PublicationStatus::Success { accept_tid };
        Ok(())
    }
    fn update_status_to_publication_failure(
        &self,
        publication: &Publication,
    ) -> Result<(), StoreError> {
        let mut inner = self.checked()?;
        let entry = inner
            .publications
            .get_mut(&publication.publication_id)
            .ok_or(StoreError)?;
        entry.1 = PublicationStatus::Failure;
        Ok(())
    }
    fn load_game_event_log_list(&self) -> Result<Vec<EventLog>, StoreError> {
        let inner = self.checked()?;
        Ok(inner
            .logs
            .values()
            .filter(|(_, s)| *s == LogStatus::Registered)
            .map(|(l, _)| l.clone())
            .take(MAX_LIST)
            .collect())
    }
    fn load_coupon_publication_list(&self) -> Result<Vec<Publication>, StoreError> {
        let inner = self.checked()?;
        Ok(inner
            .publications
            .values()
            .filter(|(_, s)| *s == PublicationStatus::Ready)
            .map(|(p, _)| *p)
            .take(MAX_LIST)
            .collect())
    }
    fn load_single_coupon_publication(
        &self,
        publication_id: i64,
        account: &Account,
    ) -> Result<i32, StoreError> {
        let inner = self.checked()?;
        Ok(match inner.publications.get(&publication_id) {
            Some((p, PublicationStatus::Ready | PublicationStatus::Failure))
                if p.account == *account =>
            {
                p.coupon_event_id
            }
            _ => 0,
        })
    }
    fn load_retry_ready_game_event_log_list(&self) -> Result<Vec<EventLog>, StoreError> {
        let inner = self.checked()?;
        Ok(inner
            .logs
            .values()
            .filter(|(_, s)| *s == LogStatus::ProcessFailure)
            .map(|(l, _)| l.clone())
            .take(MAX_LIST)
            .collect())
    }
}
