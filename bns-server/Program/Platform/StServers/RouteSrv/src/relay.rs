//! Request origin, reply return, and transaction mapping (spec §8.1).
//! Raw frames are required: `Peer::call` joins partial reply bodies.
use std::collections::HashMap;
use sts::{Frame, StartLine, Transaction};
use sts_core::{
    conn::{header, reply_error, set_header, transaction, DEFAULT_TIMEOUT},
    error::module,
    route::{Location, Target},
    StsError,
};
use tokio::{sync::mpsc, time::Instant};

pub(crate) type ConnId = u64;

pub(crate) struct Link {
    pub location: Location,
    pub connected: bool,
    pub out: mpsc::UnboundedSender<Frame>,
}

struct Pending {
    caller: ConnId,
    caller_id: u32,
    origin: String,
    deadline: Instant,
}

#[derive(Default)]
pub(crate) struct Relay {
    next_id: u32,
    pending: HashMap<(ConnId, u32), Pending>,
}

pub(crate) fn unavailable() -> StsError {
    StsError::new(49, module::ROUTE, 1160)
}

/// Router-generated errors return the request origin in `t` (spec §8.1).
pub(crate) fn error_frame(frame: &Frame, error: &StsError, server_id: u32) -> Option<Frame> {
    let id = transaction(frame)?.id;
    let mut reply = reply_error(id, error, server_id);
    if let Some(origin) = header(frame, "p") {
        set_header(&mut reply, "t", origin);
    }
    Some(reply)
}

/// Header attributes use the same `; name=value` form as STS Connect.
pub(crate) fn direct_location(value: &str) -> Option<Location> {
    match Target::parse(value.split(';').next()?.trim())? {
        Target::Direct(location) => Some(location),
        _ => None,
    }
}

impl Relay {
    pub fn forward(
        &mut self,
        links: &HashMap<ConnId, Link>,
        caller: ConnId,
        provider: ConnId,
        protocol: &str,
        mut frame: Frame,
    ) -> Result<(), StsError> {
        let source = links.get(&caller).ok_or_else(unavailable)?;
        let destination = links.get(&provider).ok_or_else(unavailable)?;
        if header(&frame, "p").is_none() {
            set_header(
                &mut frame,
                "p",
                format!("*{}; protocol={protocol}", source.location.format(0)),
            );
        }
        let mut key = None;
        if let Some(txn) = transaction(&frame) {
            if txn.complete.is_some() {
                return Err(StsError::new(10, module::ROUTE, 1170));
            }
            let id = loop {
                self.next_id = self.next_id.wrapping_add(1).max(1);
                if !self.pending.contains_key(&(provider, self.next_id)) {
                    break self.next_id;
                }
            };
            key = Some((provider, id));
            self.pending.insert(
                (provider, id),
                Pending {
                    caller,
                    caller_id: txn.id,
                    origin: header(&frame, "p").unwrap_or_default().to_owned(),
                    deadline: Instant::now() + DEFAULT_TIMEOUT,
                },
            );
            set_header(&mut frame, "s", id.to_string());
        }
        if destination.out.send(frame).is_err() {
            if let Some(key) = key {
                self.pending.remove(&key);
            }
            return Err(unavailable());
        }
        Ok(())
    }

    pub fn reply(
        &mut self,
        links: &HashMap<ConnId, Link>,
        provider: ConnId,
        mut frame: Frame,
        server_id: u32,
    ) {
        debug_assert!(matches!(frame.start, StartLine::Response { .. }));
        self.expire();
        let Some(txn) = transaction(&frame).filter(|t| t.complete.is_some()) else {
            // Spec §8.1: bad transaction reply, module 9001, line 3488.
            if let Some(link) = links.get(&provider) {
                let _ = link
                    .out
                    .send(reply_error(0, &StsError::new(4, 9001, 3488), server_id));
            }
            return;
        };
        let key = (provider, txn.id);
        let Some(pending) = self.pending.get(&key) else {
            return;
        };
        let target = header(&frame, "t").and_then(direct_location);
        let origin = direct_location(&pending.origin);
        if target.is_none() || target != origin {
            if let Some(link) = links.get(&provider) {
                let _ = link.out.send(reply_error(
                    txn.id,
                    &StsError::new(10, module::ROUTE, 1115),
                    server_id,
                ));
            }
            return;
        }
        let caller = pending.caller;
        let restored = Transaction {
            id: pending.caller_id,
            ..txn
        };
        set_header(
            &mut frame,
            "s",
            restored.to_header().expect("nonzero transaction").value,
        );
        if header(&frame, "p").is_none() {
            set_header(&mut frame, "p", pending.origin.clone());
        }
        if txn.complete == Some(true) {
            self.pending.remove(&key);
        }
        if let Some(link) = links.get(&caller) {
            let _ = link.out.send(frame);
        }
    }

    pub fn expire(&mut self) {
        let now = Instant::now();
        self.pending.retain(|_, pending| pending.deadline > now);
    }

    pub fn disconnect(&mut self, links: &HashMap<ConnId, Link>, lost: ConnId, server_id: u32) {
        self.expire();
        self.pending.retain(|(provider, _), pending| {
            if pending.caller == lost {
                return false;
            }
            if *provider != lost {
                return true;
            }
            if let Some(link) = links.get(&pending.caller) {
                // ❓ not traced: notify pending callers when their provider drops.
                let mut frame = reply_error(pending.caller_id, &unavailable(), server_id);
                set_header(&mut frame, "t", pending.origin.clone());
                let _ = link.out.send(frame);
            }
            false
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sts_core::conn::{reply_raw, request};

    struct Setup {
        links: HashMap<ConnId, Link>,
        caller: mpsc::UnboundedReceiver<Frame>,
        provider: mpsc::UnboundedReceiver<Frame>,
        relay: Relay,
    }

    fn setup() -> Setup {
        let mut links = HashMap::new();
        let (caller_out, caller) = mpsc::unbounded_channel();
        let (provider_out, provider) = mpsc::unbounded_channel();
        for (id, out) in [(1, caller_out), (2, provider_out)] {
            links.insert(
                id,
                Link {
                    location: Location {
                        addr: Some(format!("127.0.0.1:{}", 7000 + id).parse().unwrap()),
                        ..Location::default()
                    },
                    connected: true,
                    out,
                },
            );
        }
        Setup {
            links,
            caller,
            provider,
            relay: Relay::default(),
        }
    }

    fn forward(setup: &mut Setup) -> Frame {
        let mut call = request("Auth", "Test", None);
        set_header(&mut call, "s", "100");
        setup
            .relay
            .forward(&setup.links, 1, 2, "Auth", call)
            .unwrap();
        setup.provider.try_recv().unwrap()
    }

    fn reply(call: &Frame, complete: bool) -> Frame {
        let mut reply = reply_raw(
            transaction(call).unwrap().id,
            200,
            "OK",
            vec![],
            complete,
            1,
        );
        set_header(&mut reply, "t", header(call, "p").unwrap());
        reply
    }

    #[tokio::test(start_paused = true)]
    async fn partial_reply_keeps_original_deadline_and_final_removes_entry() {
        let mut setup = setup();
        let call = forward(&mut setup);
        tokio::time::advance(std::time::Duration::from_millis(24_999)).await;
        setup.relay.reply(&setup.links, 2, reply(&call, false), 9);
        assert_eq!(setup.relay.pending.len(), 1);
        assert_eq!(
            transaction(&setup.caller.try_recv().unwrap()).unwrap().id,
            100
        );
        tokio::time::advance(std::time::Duration::from_millis(1)).await;
        setup.relay.expire();
        assert!(setup.relay.pending.is_empty());
        setup.relay.reply(&setup.links, 2, reply(&call, true), 9);
        assert!(setup.caller.try_recv().is_err());
        let call = forward(&mut setup);
        setup.relay.reply(&setup.links, 2, reply(&call, true), 9);
        assert!(setup.relay.pending.is_empty());
        setup.caller.try_recv().unwrap();
        setup.relay.reply(&setup.links, 2, reply(&call, true), 9);
        assert!(setup.caller.try_recv().is_err());
    }

    #[tokio::test]
    async fn caller_disconnect_removes_entries_without_provider_error() {
        let mut setup = setup();
        forward(&mut setup);
        setup.relay.disconnect(&setup.links, 1, 9);
        assert!(setup.relay.pending.is_empty());
        assert!(setup.provider.try_recv().is_err());
        assert!(setup.caller.try_recv().is_err());
    }

    #[tokio::test]
    async fn failed_send_does_not_leave_a_pending_entry() {
        let mut setup = setup();
        setup.provider.close();
        let mut call = request("Auth", "Test", None);
        set_header(&mut call, "s", "100");
        assert_eq!(
            setup.relay.forward(&setup.links, 1, 2, "Auth", call),
            Err(unavailable())
        );
        assert!(setup.relay.pending.is_empty());
    }
}
