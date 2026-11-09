//! Fixed-size lock-free ring buffer (multi-producer, single consumer).
//!
//! The producer side runs inside an exception handler, so `push` does not
//! allocate and does not take a lock. Each slot has a sequence number
//! (Vyukov bounded queue). A full ring drops the new value and counts it.

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

struct Slot<T> {
    seq: AtomicUsize,
    val: UnsafeCell<MaybeUninit<T>>,
}

pub struct Ring<T, const N: usize> {
    slots: Box<[Slot<T>]>,
    head: AtomicUsize, // next position to pop
    tail: AtomicUsize, // next position to push
    dropped: AtomicU64,
}

// SAFETY: a slot value is only touched by the thread that owns the slot
// according to the sequence number protocol.
unsafe impl<T: Send, const N: usize> Send for Ring<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for Ring<T, N> {}

impl<T, const N: usize> Default for Ring<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Ring<T, N> {
    /// Make an empty ring. Allocates once, here. `N` must be at least 1.
    pub fn new() -> Self {
        assert!(N >= 1);
        let slots: Vec<Slot<T>> = (0..N)
            .map(|i| Slot {
                seq: AtomicUsize::new(i),
                val: UnsafeCell::new(MaybeUninit::uninit()),
            })
            .collect();
        Ring {
            slots: slots.into_boxed_slice(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            dropped: AtomicU64::new(0),
        }
    }

    /// Push a value. Returns false (and counts a drop) when the ring is full.
    pub fn push(&self, v: T) -> bool {
        let mut pos = self.tail.load(Ordering::Relaxed);
        loop {
            let slot = &self.slots[pos % N];
            let seq = slot.seq.load(Ordering::Acquire);
            let diff = seq as isize - pos as isize;
            if diff == 0 {
                match self.tail.compare_exchange_weak(
                    pos,
                    pos.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // SAFETY: we own this slot until we publish seq.
                        unsafe { (*slot.val.get()).write(v) };
                        slot.seq.store(pos.wrapping_add(1), Ordering::Release);
                        return true;
                    }
                    Err(p) => pos = p,
                }
            } else if diff < 0 {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                return false;
            } else {
                pos = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    /// Pop the oldest value. Only one thread may call this at a time.
    pub fn pop(&self) -> Option<T> {
        let pos = self.head.load(Ordering::Relaxed);
        let slot = &self.slots[pos % N];
        let seq = slot.seq.load(Ordering::Acquire);
        if seq as isize - pos.wrapping_add(1) as isize != 0 {
            return None;
        }
        // SAFETY: seq == pos+1 means the producer finished writing.
        let v = unsafe { (*slot.val.get()).assume_init_read() };
        slot.seq.store(pos.wrapping_add(N), Ordering::Release);
        self.head.store(pos.wrapping_add(1), Ordering::Relaxed);
        Some(v)
    }

    /// Number of values dropped because the ring was full.
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

impl<T, const N: usize> Drop for Ring<T, N> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_full() {
        let r: Ring<u32, 4096> = Ring::new();
        for i in 0..4096 {
            assert!(r.push(i));
        }
        assert!(!r.push(4096));
        assert_eq!(r.dropped(), 1);
        assert_eq!(r.pop(), Some(0));
        assert!(r.push(4097));
    }

    #[test]
    fn ring_order() {
        let r: Ring<u32, 8> = Ring::new();
        r.push(1);
        r.push(2);
        r.push(3);
        assert_eq!(r.pop(), Some(1));
        assert_eq!(r.pop(), Some(2));
        assert_eq!(r.pop(), Some(3));
        assert_eq!(r.pop(), None);
    }

    #[test]
    fn ring_threads() {
        use std::sync::Arc;
        let r: Arc<Ring<u32, 64>> = Arc::new(Ring::new());
        let mut hs = Vec::new();
        for t in 0..4u32 {
            let r = r.clone();
            hs.push(std::thread::spawn(move || {
                for i in 0..1000 {
                    while !r.push(t * 1000 + i) {}
                }
            }));
        }
        let mut got = 0;
        while got < 4000 {
            if r.pop().is_some() {
                got += 1;
            }
        }
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(r.pop(), None);
    }
}
