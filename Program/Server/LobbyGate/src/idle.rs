//! Idle receive state. Spec 01 §4. No packet or crypt state belongs here.
use std::{
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, ReadBuf},
    time::Instant,
};

pub const SCAN: Duration = Duration::from_secs(5);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Received,
    Watching,
    Candidate,
    Reported,
}
pub struct IdleState {
    last: Instant,
    phase: Phase,
    timeout: Duration,
}
impl IdleState {
    pub fn new(seconds: i32) -> Self {
        Self {
            last: Instant::now(),
            phase: Phase::Received,
            timeout: Duration::from_secs(seconds as u16 as u64),
        }
    }
    fn received(&mut self) {
        self.last = Instant::now();
        self.phase = Phase::Received;
    }
    /// Return true only on the report scan. Receive starts the cycle again.
    pub fn scan(&mut self) -> bool {
        let expired = self.last + self.timeout < Instant::now();
        match self.phase {
            Phase::Received => self.phase = Phase::Watching,
            Phase::Watching if expired => self.phase = Phase::Candidate,
            Phase::Candidate if expired => {
                self.phase = Phase::Reported;
                return true;
            }
            _ => (),
        }
        false
    }
}
/// Count actual receive progress, including partial frames and partial Hello.
pub struct ReceiveWatch<R> {
    pub reader: R,
    pub state: Arc<Mutex<IdleState>>,
}
impl<R: AsyncRead + Unpin> AsyncRead for ReceiveWatch<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut self.reader).poll_read(cx, buf);
        if matches!(result, Poll::Ready(Ok(()))) && buf.filled().len() > before {
            self.state.lock().unwrap().received();
        }
        result
    }
}
pub fn cadence(start: Instant, period: Duration) -> tokio::time::Interval {
    let mut ticks = tokio::time::interval_at(start + period, period);
    ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    ticks
}
#[cfg(test)]
mod tests {
    #[tokio::test(start_paused = true)]
    async fn cadence_waits_one_period_and_skips_missed_ticks() {
        use super::*;
        let start = Instant::now();
        let mut ticks = cadence(start, Duration::from_millis(100));
        assert!(
            tokio::time::timeout(Duration::from_millis(99), ticks.tick())
                .await
                .is_err()
        );
        assert_eq!(ticks.tick().await, start + Duration::from_millis(100));
        tokio::time::advance(Duration::from_millis(450)).await;
        ticks.tick().await;
        assert!(
            tokio::time::timeout(Duration::from_millis(49), ticks.tick())
                .await
                .is_err()
        );
        assert_eq!(ticks.tick().await, start + Duration::from_millis(600));
    }
    use super::*;
    #[tokio::test(start_paused = true)]
    async fn first_scan_strict_boundary_two_checks_and_receive_reset() {
        let mut state = IdleState::new(10);
        tokio::time::advance(SCAN).await;
        assert!(!state.scan()); // first scan only arms the watcher
        tokio::time::advance(SCAN).await;
        assert!(!state.scan()); // equality is not expired
        assert_eq!(state.phase, Phase::Watching);
        tokio::time::advance(SCAN).await;
        assert!(!state.scan());
        assert_eq!(state.phase, Phase::Candidate);
        state.received();
        tokio::time::advance(SCAN).await;
        assert!(!state.scan());
        tokio::time::advance(SCAN).await;
        assert!(!state.scan());
        tokio::time::advance(SCAN).await;
        assert!(!state.scan());
        tokio::time::advance(SCAN).await;
        assert!(state.scan());
        tokio::time::advance(SCAN).await;
        assert!(!state.scan()); // log once until another receive
    }
    #[tokio::test(start_paused = true)]
    async fn zero_and_low_sixteen_bits_need_three_scans() {
        for seconds in [0, 65536] {
            let mut state = IdleState::new(seconds);
            for expected in [false, false, true] {
                tokio::time::advance(SCAN).await;
                assert_eq!(state.scan(), expected);
            }
        }
    }
    #[tokio::test(start_paused = true)]
    async fn partial_receive_restarts_idle_state() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let (read, mut write) = tokio::io::duplex(8);
        let state = Arc::new(Mutex::new(IdleState::new(0)));
        let mut read = ReceiveWatch {
            reader: read,
            state: state.clone(),
        };
        tokio::time::advance(SCAN).await;
        assert!(!state.lock().unwrap().scan());
        tokio::time::advance(SCAN).await;
        assert!(!state.lock().unwrap().scan());
        write.write_all(&[1]).await.unwrap();
        assert_eq!(read.read_u8().await.unwrap(), 1);
        assert_eq!(state.lock().unwrap().phase, Phase::Received);
        assert!(!state.lock().unwrap().scan());
    }
}
