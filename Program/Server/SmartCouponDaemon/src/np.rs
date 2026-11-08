//! NP (next platform) coupon notification (spec 05-np-job.md).
//! NPServiceChannel::SendRequestEventNoti @0x14000FF10 and
//! EventNotiJob::eventNotiRes. The NP transport itself is not ported.
use crate::store::Account;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

/// Classic command id. Reward changeover uses
/// COMMAND_SRV_REWARD_REQUEST_REWARD_FOR_EVENT with nRequestFromCode 9.
pub const COMMAND_EVENT_NOTI: i32 = 2851;
pub const COMMAND_EVENT_NOTI_ALT: i32 = 2852;
pub const REWARD_REQUEST_FROM_CODE: i32 = 9;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventNoti {
    pub account: Account,
    pub coupon_event_id: i32,
    pub publication_id: i64,
    /// np-reward-changeover-enable.
    pub reward_changeover: bool,
}

impl EventNoti {
    /// pszEventId ("%d").
    pub fn event_id_text(&self) -> String {
        self.coupon_event_id.to_string()
    }
    /// pszRequestKey ("%I64d").
    pub fn request_key_text(&self) -> String {
        self.publication_id.to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NpReply {
    Success { accept_tid: i64 },
    Error { code: i32 },
    Expired,
}

/// NP error code that native handles as a publication success.
pub const SUCCESS_ERROR_CODE: i32 = 5809;

/// Error codes that do not mark the publication as failed.
pub fn keeps_publication_status(code: i32) -> bool {
    matches!(code, 42 | 5812 | 30002)
}

/// eventNotiRes classification. `None` means the command id is not an
/// event-noti reply ❓ (the job ignores it).
pub fn classify(
    status: i32,
    reason_phrase: &str,
    command: i32,
    accept_tid_field: &str,
    error_code: i32,
) -> Option<NpReply> {
    // Error 5809 takes the success path (0x14000A310); the phrase test is
    // _wcsicmp.
    let success = status == 200
        || (status == 400 && reason_phrase.eq_ignore_ascii_case("Success"))
        || error_code == SUCCESS_ERROR_CODE;
    if success {
        if command != COMMAND_EVENT_NOTI && command != COMMAND_EVENT_NOTI_ALT {
            return None;
        }
        Some(NpReply::Success {
            accept_tid: wtoi64(accept_tid_field),
        })
    } else {
        Some(NpReply::Error { code: error_code })
    }
}

/// `_wtoi64`: leading spaces, optional sign, digits; stops at the first other char.
pub fn wtoi64(text: &str) -> i64 {
    let t = text.trim_start();
    let (neg, digits) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let mut n: i64 = 0;
    for b in digits.bytes().take_while(u8::is_ascii_digit) {
        n = n.wrapping_mul(10).wrapping_add((b - b'0') as i64);
    }
    if neg {
        n.wrapping_neg()
    } else {
        n
    }
}

pub type NpFuture<'a> = Pin<Box<dyn Future<Output = Result<NpReply, ()>> + Send + 'a>>;

/// `Err(())` means no NP channel or a failed send.
pub trait Np: Send + Sync + 'static {
    fn event_noti(&self, request: EventNoti) -> NpFuture<'_>;
}

/// No NP channel (np-enabled off or not connected).
pub struct NoNp;
impl Np for NoNp {
    fn event_noti(&self, _request: EventNoti) -> NpFuture<'_> {
        Box::pin(async { Err(()) })
    }
}

/// Scripted NP for tests and local runs. It records every request.
#[derive(Default)]
pub struct ScriptedNp {
    pub replies: Mutex<Vec<Result<NpReply, ()>>>,
    pub sent: Mutex<Vec<EventNoti>>,
}
impl ScriptedNp {
    pub fn push(&self, reply: Result<NpReply, ()>) {
        self.replies.lock().unwrap().push(reply);
    }
    pub fn sent(&self) -> Vec<EventNoti> {
        self.sent.lock().unwrap().clone()
    }
}
impl Np for ScriptedNp {
    fn event_noti(&self, request: EventNoti) -> NpFuture<'_> {
        self.sent.lock().unwrap().push(request);
        let reply = {
            let mut replies = self.replies.lock().unwrap();
            if replies.is_empty() {
                Err(())
            } else {
                replies.remove(0)
            }
        };
        Box::pin(async move { reply })
    }
}
