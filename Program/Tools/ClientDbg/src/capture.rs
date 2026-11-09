//! Capture spec: what to record at each breakpoint hit.
//!
//! Grammar (items split by `,`):
//! * `regs`
//! * `stack:N` — N dwords from esp (N <= 64)
//! * `mem:REG+OFF:LEN` — LEN bytes at REG+OFF (LEN <= 256)
//! * `deref:REG+OFF` — read a pointer at REG+OFF, then 64 bytes at the pointer
//!
//! `run` has no allocation and no OS calls, so it is safe inside an
//! exception handler. Memory reads go through the `MemRead` trait.

use crate::proto::RegSet;
use std::fmt;

pub const MAX_ITEMS: usize = 8;
pub const MAX_STACK_DWORDS: u32 = 64;
pub const MAX_MEM_LEN: u32 = 256;
pub const DEREF_LEN: usize = 64;
/// Total bytes of captured data per hit.
pub const CAP_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    Eax,
    Ecx,
    Edx,
    Ebx,
    Esp,
    Ebp,
    Esi,
    Edi,
    Eip,
}

impl Reg {
    pub fn name(self) -> &'static str {
        match self {
            Reg::Eax => "eax",
            Reg::Ecx => "ecx",
            Reg::Edx => "edx",
            Reg::Ebx => "ebx",
            Reg::Esp => "esp",
            Reg::Ebp => "ebp",
            Reg::Esi => "esi",
            Reg::Edi => "edi",
            Reg::Eip => "eip",
        }
    }

    fn from_name(s: &str) -> Option<Reg> {
        Some(match s {
            "eax" => Reg::Eax,
            "ecx" => Reg::Ecx,
            "edx" => Reg::Edx,
            "ebx" => Reg::Ebx,
            "esp" => Reg::Esp,
            "ebp" => Reg::Ebp,
            "esi" => Reg::Esi,
            "edi" => Reg::Edi,
            "eip" => Reg::Eip,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Regs,
    Stack(u32),
    Mem { reg: Reg, off: i32, len: u32 },
    Deref { reg: Reg, off: i32 },
}

fn fmt_off(f: &mut fmt::Formatter<'_>, off: i32) -> fmt::Result {
    if off < 0 {
        write!(f, "-{:#x}", -(off as i64))
    } else {
        write!(f, "+{off:#x}")
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Item::Regs => write!(f, "regs"),
            Item::Stack(n) => write!(f, "stack:{n}"),
            Item::Mem { reg, off, len } => {
                write!(f, "mem:{}", reg.name())?;
                fmt_off(f, off)?;
                write!(f, ":{len}")
            }
            Item::Deref { reg, off } => {
                write!(f, "deref:{}", reg.name())?;
                fmt_off(f, off)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    BadItem(String),
    BadRegister(String),
    BadNumber(String),
    TooLong { what: &'static str, max: u32 },
    TooManyItems,
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureError::BadItem(s) => write!(f, "bad capture item: {s}"),
            CaptureError::BadRegister(s) => write!(f, "bad register: {s}"),
            CaptureError::BadNumber(s) => write!(f, "bad number: {s}"),
            CaptureError::TooLong { what, max } => write!(f, "{what} is more than {max}"),
            CaptureError::TooManyItems => write!(f, "more than {MAX_ITEMS} items"),
        }
    }
}

impl std::error::Error for CaptureError {}

fn num(s: &str) -> Result<u32, CaptureError> {
    let s = s.trim();
    let r = match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(h) => u32::from_str_radix(h, 16),
        None => s.parse::<u32>(),
    };
    r.map_err(|_| CaptureError::BadNumber(s.to_string()))
}

/// Parse `REG`, `REG+OFF` or `REG-OFF`.
fn reg_off(s: &str) -> Result<(Reg, i32), CaptureError> {
    let s = s.trim().to_ascii_lowercase();
    let (name, sign, rest) = match s.find(['+', '-']) {
        Some(i) => (
            &s[..i],
            if s.as_bytes()[i] == b'-' { -1i64 } else { 1 },
            &s[i + 1..],
        ),
        None => (s.as_str(), 1, ""),
    };
    let reg =
        Reg::from_name(name.trim()).ok_or_else(|| CaptureError::BadRegister(name.to_string()))?;
    let off = if rest.is_empty() && name.len() == s.len() {
        0
    } else {
        num(rest)? as i64 * sign
    };
    let off = i32::try_from(off).map_err(|_| CaptureError::BadNumber(s.clone()))?;
    Ok((reg, off))
}

/// Parse a capture spec. An empty spec gives an empty list.
pub fn parse(spec: &str) -> Result<Vec<Item>, CaptureError> {
    let mut out = Vec::new();
    for raw in spec.split(',') {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let item = if raw == "regs" {
            Item::Regs
        } else if let Some(n) = raw.strip_prefix("stack:") {
            let n = num(n)?;
            if n > MAX_STACK_DWORDS {
                return Err(CaptureError::TooLong {
                    what: "stack count",
                    max: MAX_STACK_DWORDS,
                });
            }
            Item::Stack(n)
        } else if let Some(r) = raw.strip_prefix("mem:") {
            let (ro, len) = r
                .rsplit_once(':')
                .ok_or_else(|| CaptureError::BadItem(raw.to_string()))?;
            let (reg, off) = reg_off(ro)?;
            let len = num(len)?;
            if len > MAX_MEM_LEN {
                return Err(CaptureError::TooLong {
                    what: "mem length",
                    max: MAX_MEM_LEN,
                });
            }
            Item::Mem { reg, off, len }
        } else if let Some(r) = raw.strip_prefix("deref:") {
            let (reg, off) = reg_off(r)?;
            Item::Deref { reg, off }
        } else {
            return Err(CaptureError::BadItem(raw.to_string()));
        };
        out.push(item);
        if out.len() > MAX_ITEMS {
            return Err(CaptureError::TooManyItems);
        }
    }
    Ok(out)
}

pub fn reg_value(regs: &RegSet, r: Reg) -> u32 {
    match r {
        Reg::Eax => regs.eax,
        Reg::Ecx => regs.ecx,
        Reg::Edx => regs.edx,
        Reg::Ebx => regs.ebx,
        Reg::Esp => regs.esp,
        Reg::Ebp => regs.ebp,
        Reg::Esi => regs.esi,
        Reg::Edi => regs.edi,
        Reg::Eip => regs.eip,
    }
}

/// Guarded memory reader. Returns false when the range is not readable.
pub trait MemRead {
    fn read(&self, addr: u32, out: &mut [u8]) -> bool;
}

pub const ST_OK: u8 = 0;
pub const ST_UNREADABLE: u8 = 1;
pub const ST_TRUNCATED: u8 = 2;

/// One captured item.
#[derive(Debug, Clone, Copy)]
pub struct CapEnt {
    pub item: Item,
    pub status: u8,
    /// For `deref`: the pointer value. For `mem` and `stack`: the start address.
    pub ptr: u32,
    pub off: u16,
    pub len: u16,
}

/// Fixed-size capture result. No heap use.
pub struct CapBuf {
    pub data: [u8; CAP_BYTES],
    pub used: usize,
    pub ents: [Option<CapEnt>; MAX_ITEMS],
    pub n: usize,
}

impl Default for CapBuf {
    fn default() -> Self {
        CapBuf {
            data: [0; CAP_BYTES],
            used: 0,
            ents: [None; MAX_ITEMS],
            n: 0,
        }
    }
}

impl CapBuf {
    pub fn entries(&self) -> impl Iterator<Item = (&CapEnt, &[u8])> {
        self.ents[..self.n].iter().flatten().map(move |e| {
            let a = e.off as usize;
            (e, &self.data[a..a + e.len as usize])
        })
    }

    /// Reserve `want` bytes. Returns (offset, granted length, status).
    fn reserve(&mut self, want: usize) -> (usize, usize, u8) {
        let free = CAP_BYTES - self.used;
        let got = want.min(free);
        let off = self.used;
        self.used += got;
        (off, got, if got < want { ST_TRUNCATED } else { ST_OK })
    }

    fn add(&mut self, e: CapEnt) {
        if self.n < MAX_ITEMS {
            self.ents[self.n] = Some(e);
            self.n += 1;
        }
    }
}

/// Run all capture items for one hit. `Item::Regs` makes no entry (the event
/// always has the registers).
pub fn run(items: &[Item], regs: &RegSet, rd: &dyn MemRead, out: &mut CapBuf) {
    for &item in items {
        let (start, want) = match item {
            Item::Regs => continue,
            Item::Stack(n) => (regs.esp, n as usize * 4),
            Item::Mem { reg, off, len } => {
                (reg_value(regs, reg).wrapping_add(off as u32), len as usize)
            }
            Item::Deref { reg, off } => {
                let at = reg_value(regs, reg).wrapping_add(off as u32);
                let mut p = [0u8; 4];
                if !rd.read(at, &mut p) {
                    out.add(CapEnt {
                        item,
                        status: ST_UNREADABLE,
                        ptr: 0,
                        off: 0,
                        len: 0,
                    });
                    continue;
                }
                (u32::from_le_bytes(p), DEREF_LEN)
            }
        };
        let (off, got, mut status) = out.reserve(want);
        if got > 0 && !rd.read(start, &mut out.data[off..off + got]) {
            status = ST_UNREADABLE;
            out.data[off..off + got].fill(0);
        }
        out.add(CapEnt {
            item,
            status,
            ptr: start,
            off: off as u16,
            len: got as u16,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_parse() {
        let v = parse("regs,stack:8,mem:ecx+0x10:64,deref:esp+4").unwrap();
        assert_eq!(
            v,
            vec![
                Item::Regs,
                Item::Stack(8),
                Item::Mem {
                    reg: Reg::Ecx,
                    off: 0x10,
                    len: 64
                },
                Item::Deref {
                    reg: Reg::Esp,
                    off: 4
                },
            ]
        );
    }

    #[test]
    fn capture_limits() {
        assert!(matches!(
            parse("mem:ecx:257"),
            Err(CaptureError::TooLong { .. })
        ));
        assert!(parse("mem:ecx:256").is_ok());
        assert!(parse("stack:65").is_err());
        assert!(parse("mem:foo+1:4").is_err());
        assert!(parse("nope").is_err());
        assert!(parse("regs,regs,regs,regs,regs,regs,regs,regs,regs").is_err());
    }

    #[test]
    fn capture_offsets_and_display() {
        let v = parse("deref:ebp-8,mem:esi:4,deref:eax").unwrap();
        assert_eq!(
            v[0],
            Item::Deref {
                reg: Reg::Ebp,
                off: -8
            }
        );
        assert_eq!(
            v[1],
            Item::Mem {
                reg: Reg::Esi,
                off: 0,
                len: 4
            }
        );
        assert_eq!(
            v[2],
            Item::Deref {
                reg: Reg::Eax,
                off: 0
            }
        );
        assert_eq!(v[0].to_string(), "deref:ebp-0x8");
        assert_eq!(parse(&v[0].to_string()).unwrap()[0], v[0]);
    }

    struct Flat(u32, Vec<u8>);
    impl MemRead for Flat {
        fn read(&self, addr: u32, out: &mut [u8]) -> bool {
            let Some(s) = addr.checked_sub(self.0) else {
                return false;
            };
            let s = s as usize;
            match self.1.get(s..s + out.len()) {
                Some(b) => {
                    out.copy_from_slice(b);
                    true
                }
                None => false,
            }
        }
    }

    #[test]
    fn capture_run() {
        let mut mem = vec![0u8; 0x100];
        mem[0x10..0x14].copy_from_slice(&0x1040u32.to_le_bytes());
        mem[0x40] = 0xaa;
        let rd = Flat(0x1000, mem);
        let regs = RegSet {
            ecx: 0x1000,
            esp: 0x1010,
            ..Default::default()
        };
        let items = parse("stack:1,mem:ecx+0x40:2,deref:esp,mem:ecx+0x1000:4").unwrap();
        let mut cb = CapBuf::default();
        run(&items, &regs, &rd, &mut cb);
        let e: Vec<_> = cb.entries().collect();
        assert_eq!(e.len(), 4);
        assert_eq!(e[0].1, &0x1040u32.to_le_bytes());
        assert_eq!(e[1].1, &[0xaa, 0]);
        assert_eq!(e[2].0.ptr, 0x1040);
        assert_eq!(e[2].0.status, ST_OK);
        assert_eq!(e[2].1.len(), DEREF_LEN);
        assert_eq!(e[2].1[0], 0xaa);
        assert_eq!(e[3].0.status, ST_UNREADABLE);
    }
}
