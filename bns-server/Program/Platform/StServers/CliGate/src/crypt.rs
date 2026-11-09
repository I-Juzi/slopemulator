//! RC4 client link (spec §6). After `enable(key)` every byte in both directions
//! goes through RC4 with the same key (two cipher states, no dropped bytes).

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use sts::rc4::Rc4;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct CryptStream<S> {
    inner: S,
    /// (send, receive) after `enable`.
    ciphers: Option<(Rc4, Rc4)>,
    /// Bytes that a reader took from the socket before `enable`. They are served first.
    unread: Vec<u8>,
    unread_pos: usize,
    /// Encrypted bytes that the socket did not take yet.
    out: Vec<u8>,
    out_pos: usize,
}

impl<S: AsyncRead + AsyncWrite + Unpin> CryptStream<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            ciphers: None,
            unread: Vec::new(),
            unread_pos: 0,
            out: Vec::new(),
            out_pos: 0,
        }
    }

    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Switches RC4 on. A bad key length (0 or over 256) leaves the stream plain.
    /// Bytes given with `restore_unread` before this call are decrypted here.
    pub fn enable(&mut self, key: &[u8]) {
        let (Some(send), Some(mut recv)) = (Rc4::new(key), Rc4::new(key)) else {
            return;
        };
        recv.apply(&mut self.unread[self.unread_pos..]);
        self.ciphers = Some((send, recv));
    }

    pub fn is_enabled(&self) -> bool {
        self.ciphers.is_some()
    }

    /// Puts back the bytes that a `FrameReader` read but did not use (`into_parts`).
    /// Call it before `enable`: then the bytes count as cipher text.
    pub fn restore_unread(&mut self, bytes: Vec<u8>) {
        self.unread.drain(..self.unread_pos);
        self.unread_pos = 0;
        let mut bytes = bytes;
        if let Some((_, recv)) = &mut self.ciphers {
            recv.apply(&mut bytes);
        }
        bytes.extend_from_slice(&self.unread);
        self.unread = bytes;
    }

    fn drain(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        while self.out_pos < self.out.len() {
            match Pin::new(&mut self.inner).poll_write(cx, &self.out[self.out_pos..]) {
                Poll::Ready(Ok(0)) => return Poll::Ready(Err(io::ErrorKind::WriteZero.into())),
                Poll::Ready(Ok(n)) => self.out_pos += n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        self.out.clear();
        self.out_pos = 0;
        Poll::Ready(Ok(()))
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for CryptStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.unread_pos < this.unread.len() {
            let n = buf.remaining().min(this.unread.len() - this.unread_pos);
            buf.put_slice(&this.unread[this.unread_pos..this.unread_pos + n]);
            this.unread_pos += n;
            if this.unread_pos == this.unread.len() {
                this.unread.clear();
                this.unread_pos = 0;
            }
            return Poll::Ready(Ok(()));
        }
        let before = buf.filled().len();
        let r = Pin::new(&mut this.inner).poll_read(cx, buf);
        if let (Poll::Ready(Ok(())), Some((_, recv))) = (&r, &mut this.ciphers) {
            recv.apply(&mut buf.filled_mut()[before..]);
        }
        r
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for CryptStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if data.is_empty() {
            return Poll::Ready(Ok(0));
        }
        match this.drain(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }
        let start = this.out.len();
        this.out.extend_from_slice(data);
        if let Some((send, _)) = &mut this.ciphers {
            send.apply(&mut this.out[start..]);
        }
        // The cipher state moved on, so the bytes count as accepted from here.
        if let Poll::Ready(Err(e)) = this.drain(cx) {
            return Poll::Ready(Err(e));
        }
        Poll::Ready(Ok(data.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        match this.drain(cx) {
            Poll::Ready(Ok(())) => Pin::new(&mut this.inner).poll_flush(cx),
            other => other,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        match this.drain(cx) {
            Poll::Ready(Ok(())) => Pin::new(&mut this.inner).poll_shutdown(cx),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    const KEY: [u8; 16] = [7, 1, 9, 3, 5, 2, 8, 4, 6, 0, 11, 13, 17, 19, 23, 29];

    #[tokio::test]
    async fn plain_until_enabled_then_rc4_both_ways() {
        let (a, mut b) = tokio::io::duplex(1024);
        let mut s = CryptStream::new(a);
        s.write_all(b"clear").await.unwrap();
        s.flush().await.unwrap();
        let mut got = [0u8; 5];
        b.read_exact(&mut got).await.unwrap();
        assert_eq!(&got, b"clear");

        s.enable(&KEY);
        s.write_all(b"secret one").await.unwrap();
        s.write_all(b" and two").await.unwrap();
        s.flush().await.unwrap();
        let mut wire = [0u8; 18];
        b.read_exact(&mut wire).await.unwrap();
        assert_ne!(&wire[..6], b"secret");
        Rc4::new(&KEY).unwrap().apply(&mut wire);
        assert_eq!(&wire, b"secret one and two");

        let mut peer = Rc4::new(&KEY).unwrap();
        let mut msg = *b"from peer";
        peer.apply(&mut msg);
        b.write_all(&msg).await.unwrap();
        let mut back = [0u8; 9];
        s.read_exact(&mut back).await.unwrap();
        assert_eq!(&back, b"from peer");
    }

    #[tokio::test]
    async fn unread_bytes_are_decrypted_on_enable() {
        let (a, _b) = tokio::io::duplex(64);
        let mut s = CryptStream::new(a);
        let mut cipher = *b"leftover";
        Rc4::new(&KEY).unwrap().apply(&mut cipher);
        s.restore_unread(cipher.to_vec());
        s.enable(&KEY);
        let mut got = [0u8; 8];
        s.read_exact(&mut got).await.unwrap();
        assert_eq!(&got, b"leftover");
    }

    #[tokio::test]
    async fn bad_key_keeps_stream_plain() {
        let (a, _b) = tokio::io::duplex(64);
        let mut s = CryptStream::new(a);
        s.enable(&[]);
        assert!(!s.is_enabled());
    }
}
