//! Reply buffer of one procedure call (DbSrv sub_1400D2F70, sub_1400D30B0).
//!
//! In `ExecBatched` mode, the buffer is sent as a partial reply before a
//! piece is added when buffer + piece ≥ 0xE666 bytes. The final reply carries
//! the rest.

/// Partial-reply threshold (58982 bytes).
pub const CHUNK_LIMIT: usize = 0xE666;

/// Receives the body of each partial reply, in order.
pub type Sink = Box<dyn FnMut(Vec<u8>) + Send>;

pub struct Output {
    pub buf: Vec<u8>,
    /// `Some` in `ExecBatched` mode.
    sink: Option<Sink>,
    /// Number of partial replies sent.
    pub partials: u32,
}

impl Output {
    pub fn new(sink: Option<Sink>) -> Self {
        Self { buf: Vec::new(), sink, partials: 0 }
    }

    fn flush_if(&mut self, extra: usize) {
        if let Some(sink) = &mut self.sink {
            if self.buf.len() + extra >= CHUNK_LIMIT {
                sink(std::mem::take(&mut self.buf));
                self.partials += 1;
            }
        }
    }

    /// Adds one raw piece (tag, column name).
    pub fn raw(&mut self, piece: &str) {
        self.flush_if(piece.len());
        self.buf.extend_from_slice(piece.as_bytes());
    }

    /// Adds text with `&` → `&amp;` and `<` → `&lt;`, one byte per piece.
    /// For `&` the check uses length + 5 (sub_1400D30B0).
    pub fn text(&mut self, s: &str) {
        for &b in s.as_bytes() {
            match b {
                b'&' => {
                    self.flush_if(5);
                    self.buf.extend_from_slice(b"&amp;");
                }
                b'<' => self.raw("&lt;"),
                _ => {
                    self.flush_if(1);
                    self.buf.push(b);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_only_amp_and_lt() {
        let mut o = Output::new(None);
        o.text("a&b<c>\"d'");
        assert_eq!(o.buf, b"a&amp;b&lt;c>\"d'");
    }

    #[test]
    fn splits_before_limit() {
        let got = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let g = got.clone();
        let mut o = Output::new(Some(Box::new(move |b| g.lock().unwrap().push(b.len()))));
        o.raw(&"x".repeat(CHUNK_LIMIT - 3));
        o.raw("<Row>");
        assert_eq!((o.partials, got.lock().unwrap().clone()), (1, vec![CHUNK_LIMIT - 3]));
        assert_eq!(o.buf, b"<Row>");
        let mut o = Output::new(None);
        o.raw(&"x".repeat(CHUNK_LIMIT * 2));
        assert_eq!(o.partials, 0);
    }
}
