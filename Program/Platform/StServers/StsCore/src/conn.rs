//! Async STS connection: framing, transaction ids, request/reply matching.
//!
//! A request carries `s:<id>`; its reply carries `s:<id>R` (final) or
//! `s:<id>r<seq>` (partial). A message without `s` is one-way.
//! Error replies: status 400, reason = error name, body `<Error …/>`.

use crate::{error::*, xml::Element};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use sts::{Codec, Frame, Header, StartLine, Transaction};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::{mpsc, oneshot},
};

/// Default forward timeout seen in CliGate (25000 ms).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(25_000);

pub fn header<'a>(f: &'a Frame, name: &str) -> Option<&'a str> {
    f.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)).map(|h| h.value.as_str())
}

pub fn headers_all<'a>(f: &'a Frame, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    f.headers.iter().filter(move |h| h.name.eq_ignore_ascii_case(name)).map(|h| h.value.as_str())
}

pub fn set_header(f: &mut Frame, name: &str, value: impl Into<String>) {
    f.headers.retain(|h| !h.name.eq_ignore_ascii_case(name));
    f.headers.push(Header { name: name.into(), value: value.into() });
}

pub fn transaction(f: &Frame) -> Option<Transaction> {
    header(f, "s").and_then(|v| Transaction::parse(v).ok())
}

pub fn request(protocol: &str, command: &str, body: Option<&Element>) -> Frame {
    Frame {
        start: StartLine::Request { protocol: protocol.into(), command: command.into() },
        headers: vec![],
        body: body.map(|b| b.to_xml().into_bytes()).unwrap_or_default(),
    }
}

/// Default reason text for a status (`sub_1400F2A80`).
pub fn default_reason(status: u32) -> &'static str {
    match status {
        0..=199 => "Informational",
        200..=299 => "OK",
        300..=399 => "Redirection",
        400..=499 => "Bad Request",
        _ => "Server Error",
    }
}

pub fn reply_ok(id: u32, body: Option<&Element>) -> Frame {
    let mut f = Frame {
        start: StartLine::Response { status: 200, reason: "OK".into() },
        headers: vec![],
        body: body.map(|b| b.to_xml().into_bytes()).unwrap_or_default(),
    };
    if id != 0 {
        f.headers.push(Transaction { id, complete: Some(true), sequence: 0 }.to_header().expect("id != 0"));
    }
    f
}

/// Reply with a raw body. `complete = false` gives a partial reply `s:<id>r<seq>`;
/// a final reply after partials carries `R<seq>` (StsInetSocket sub_1400DF2D0,
/// sequence counter in sub_1400EC5D0: partials 1, 2, …; final = count + 1).
pub fn reply_raw(id: u32, status: u32, reason: &str, body: Vec<u8>, complete: bool, sequence: u32) -> Frame {
    let mut f = Frame { start: StartLine::Response { status, reason: reason.into() }, headers: vec![], body };
    if id != 0 {
        f.headers.push(Transaction { id, complete: Some(complete), sequence }.to_header().expect("id != 0"));
    }
    f
}

pub fn reply_error(id: u32, err: &StsError, local_server: u32) -> Frame {
    let mut f = Frame {
        start: StartLine::Response { status: 400, reason: err.name().into() },
        headers: vec![],
        body: err.to_element(local_server).to_xml().into_bytes(),
    };
    if id != 0 {
        f.headers.push(Transaction { id, complete: Some(true), sequence: 0 }.to_header().expect("id != 0"));
    }
    f
}

/// Turns a reply into `Ok(body element or None)` or the carried error.
pub fn reply_result(f: &Frame) -> Result<Option<Element>, StsError> {
    let StartLine::Response { status, reason } = &f.start else {
        return Err(StsError::new(ERR_BAD_DATA, 0, 0));
    };
    let body = if f.body.is_empty() { None } else { Some(Element::parse(&f.body).map_err(|_| StsError::new(ERR_BAD_DATA, 0, 0))?) };
    if *status < 300 {
        return Ok(body);
    }
    Err(match body {
        Some(e) if e.name == "Error" => StsError::from_element(&e),
        _ => StsError::new(error_code(reason).unwrap_or(ERR_BAD_DATA), 0, 0),
    })
}

/// Reads frames from a byte stream.
pub struct FrameReader<R> {
    inner: R,
    buf: Vec<u8>,
    codec: Codec,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner, buf: Vec::new(), codec: Codec::default() }
    }

    /// `Ok(None)` = clean end of stream.
    pub async fn next(&mut self) -> std::io::Result<Option<Frame>> {
        loop {
            match self.codec.decode(&self.buf) {
                Ok(Some((frame, used))) => {
                    self.buf.drain(..used);
                    return Ok(Some(frame));
                }
                Ok(None) => {}
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            }
            let mut chunk = [0u8; 8192];
            let n = self.inner.read(&mut chunk).await?;
            if n == 0 {
                return if self.buf.is_empty() {
                    Ok(None)
                } else {
                    Err(std::io::ErrorKind::UnexpectedEof.into())
                };
            }
            self.buf.extend_from_slice(&chunk[..n]);
        }
    }

    pub fn into_parts(self) -> (R, Vec<u8>) {
        (self.inner, self.buf)
    }
}

pub async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, f: &Frame) -> std::io::Result<()> {
    let bytes = Codec::default().encode(f).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    w.write_all(&bytes).await
}

/// Waiting request: reply channel and the bodies of partial replies so far.
type Pending = Arc<Mutex<HashMap<u32, (oneshot::Sender<Frame>, Vec<u8>)>>>;

/// A connection that can send requests and receive incoming requests.
/// Cloning shares the same socket.
#[derive(Clone)]
pub struct Peer {
    out: mpsc::UnboundedSender<Frame>,
    pending: Pending,
    next_id: Arc<AtomicU32>,
}

/// Incoming request (or one-way message) from the peer.
pub struct Incoming {
    pub frame: Frame,
    pub peer: Peer,
}

impl Incoming {
    pub fn id(&self) -> u32 {
        transaction(&self.frame).map_or(0, |t| t.id)
    }
    pub fn command(&self) -> (&str, &str) {
        match &self.frame.start {
            StartLine::Request { protocol, command } => (protocol, command),
            _ => ("", ""),
        }
    }
    pub fn body(&self) -> Option<Element> {
        Element::parse(&self.frame.body).ok()
    }
    pub fn reply(&self, body: Option<&Element>) {
        if self.id() != 0 {
            self.peer.send(reply_ok(self.id(), body));
        }
    }
    pub fn reply_error(&self, err: &StsError, local_server: u32) {
        if self.id() != 0 {
            self.peer.send(reply_error(self.id(), err, local_server));
        }
    }
    /// Partial reply number `sequence` (1, 2, …).
    pub fn reply_partial(&self, body: Vec<u8>, sequence: u32) {
        if self.id() != 0 {
            self.peer.send(reply_raw(self.id(), 200, "OK", body, false, sequence));
        }
    }
}

impl Peer {
    /// Starts reader and writer tasks. Incoming requests go to the returned
    /// channel. The channel closes when the socket closes.
    pub fn spawn<S>(stream: S) -> (Self, mpsc::UnboundedReceiver<Incoming>)
    where
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        let (rd, mut wr) = tokio::io::split(stream);
        let (out, mut out_rx) = mpsc::unbounded_channel::<Frame>();
        let (in_tx, in_rx) = mpsc::unbounded_channel();
        let peer = Self { out, pending: Pending::default(), next_id: Arc::new(AtomicU32::new(1)) };
        tokio::spawn(async move {
            while let Some(f) = out_rx.recv().await {
                if write_frame(&mut wr, &f).await.is_err() {
                    break;
                }
            }
            let _ = wr.shutdown().await;
        });
        let p = peer.clone();
        tokio::spawn(async move {
            let mut reader = FrameReader::new(rd);
            while let Ok(Some(frame)) = reader.next().await {
                p.dispatch(frame, &in_tx);
            }
            p.pending.lock().unwrap().clear();
        });
        (peer, in_rx)
    }

    fn dispatch(&self, frame: Frame, in_tx: &mpsc::UnboundedSender<Incoming>) {
        if let StartLine::Response { .. } = frame.start {
            if let Some(t) = transaction(&frame) {
                let mut pending = self.pending.lock().unwrap();
                if t.complete == Some(true) {
                    if let Some((tx, mut acc)) = pending.remove(&t.id) {
                        let mut frame = frame;
                        if !acc.is_empty() {
                            acc.extend_from_slice(&frame.body);
                            frame.body = acc;
                        }
                        let _ = tx.send(frame);
                    }
                } else if t.complete == Some(false) {
                    // Partial reply (DbSrv ExecBatched): the bodies are joined ❓
                    // (receiver side not traced).
                    if let Some((_, acc)) = pending.get_mut(&t.id) {
                        acc.extend_from_slice(&frame.body);
                    }
                }
            }
            return;
        }
        let _ = in_tx.send(Incoming { frame, peer: self.clone() });
    }

    pub fn send(&self, f: Frame) {
        let _ = self.out.send(f);
    }

    pub fn is_closed(&self) -> bool {
        self.out.is_closed()
    }

    /// Sends a request and waits for the final reply.
    /// Timeout → `ErrTimeout`; closed socket → `ErrCanceled` (1).
    pub async fn call(&self, mut f: Frame, timeout: Duration) -> Result<Frame, StsError> {
        let id = loop {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            if id != 0 {
                break id;
            }
        };
        set_header(&mut f, "s", id.to_string());
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, (tx, Vec::new()));
        self.send(f);
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(reply)) => Ok(reply),
            Ok(Err(_)) => Err(StsError::new(1, 0, 0)),
            Err(_) => {
                self.pending.lock().unwrap().remove(&id);
                Err(StsError::new(ERR_TIMEOUT, 0, 0))
            }
        }
    }

    /// `call` and decode the reply body or error.
    pub async fn call_xml(&self, f: Frame, timeout: Duration) -> Result<Option<Element>, StsError> {
        reply_result(&self.call(f, timeout).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn call_and_reply() {
        let (a, b) = tokio::io::duplex(4096);
        let (client, _) = Peer::spawn(a);
        let (_server, mut rx) = Peer::spawn(b);
        tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                match req.command() {
                    ("Test", "Ok") => req.reply(Some(&Element::int("V", 7))),
                    _ => req.reply_error(&StsError::new(ERR_BAD_PARAM, 2, 1115), 9),
                }
            }
        });
        let r = client.call_xml(request("Test", "Ok", None), DEFAULT_TIMEOUT).await.unwrap().unwrap();
        assert_eq!(r.text, "7");
        let e = client.call_xml(request("Test", "Bad", None), DEFAULT_TIMEOUT).await.unwrap_err();
        assert_eq!((e.code, e.server, e.module, e.line), (10, 9, 2, 1115));
    }

    #[tokio::test]
    async fn partial_replies_are_joined() {
        let (a, b) = tokio::io::duplex(4096);
        let (client, _) = Peer::spawn(a);
        let (_server, mut rx) = Peer::spawn(b);
        tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                req.reply_partial(b"<V>1".to_vec(), 1);
                req.reply_partial(b"2".to_vec(), 2);
                req.peer.send(reply_raw(req.id(), 200, "OK", b"3</V>\n".to_vec(), true, 3));
            }
        });
        let r = client.call_xml(request("Test", "P", None), DEFAULT_TIMEOUT).await.unwrap().unwrap();
        assert_eq!(r.text, "123");
    }
}
