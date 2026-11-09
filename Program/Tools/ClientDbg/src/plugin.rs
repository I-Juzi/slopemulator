//! Plugin side (runs inside `Client.exe`, 32-bit Windows only).
//!
//! * `GPluginInfo` is read by the bnspatch plugin loader (SDK 3.1.0.0).
//! * `oep_notify` starts one worker thread. It connects to the host, sends
//!   events from the ring buffer, runs commands and re-applies hardware
//!   breakpoints to new threads.
//! * The vectored exception handler runs on client threads. It uses only
//!   the lock-free ring and fixed buffers.

use crate::capture::{self, CapBuf, Item, ST_OK, ST_TRUNCATED};
use crate::dr;
use crate::hex;
use crate::ini;
use crate::proto::{
    self, CapOut, Command, Event, Hello, HelloMsg, Kind, RebaseInfo, RegSet, Reply, Request,
};
use crate::ring::Ring;
use crate::win;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::panic::catch_unwind;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering};
use std::sync::{Once, OnceLock};
use std::time::{Duration, Instant};
use windows_sys::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, EXCEPTION_POINTERS,
};

const RECONNECT_MS: u64 = 5000;
const REAPPLY_MS: u64 = 1000;
const RING_SLOTS: usize = 4096;
const MAX_READ: u32 = 65536;
const MAX_WRITE: usize = 4096;
const EXCEPTION_SINGLE_STEP: i32 = 0x8000_0004u32 as i32;
const CONTINUE_SEARCH: i32 = 0;
const CONTINUE_EXECUTION: i32 = -1;
const RESUME_FLAG: u32 = 0x1_0000;

// ---------------------------------------------------------------------------
// Plugin info for the loader

#[repr(C)]
#[derive(Clone, Copy)]
pub union Version {
    pub f: VersionFields,
    pub version: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VersionFields {
    pub major: u16,
    pub minor: u16,
    pub build: u16,
    pub revision: u16,
}

#[repr(C)]
pub struct PluginInfo {
    pub sdk_version: Version,
    pub hide_from_peb: u8,
    pub erase_pe_header: u8,
    pub init: extern "C" fn(Version) -> i32,
    pub oep_notify: extern "C" fn(Version),
    pub priority: i32,
    pub target_apps: *const u16,
}

// SAFETY: the struct is read-only and holds only function pointers and a
// pointer to static data.
unsafe impl Sync for PluginInfo {}

/// UTF-16 `Client.exe` with a terminating zero.
static TARGET_APPS: [u16; 11] = {
    let s = b"Client.exe\0";
    let mut w = [0u16; 11];
    let mut i = 0;
    while i < 11 {
        w[i] = s[i] as u16;
        i += 1;
    }
    w
};

#[no_mangle]
#[allow(non_upper_case_globals)]
pub static GPluginInfo: PluginInfo = PluginInfo {
    sdk_version: Version {
        version: 3 | (1 << 16),
    },
    hide_from_peb: 0,
    erase_pe_header: 0,
    init: plugin_init,
    oep_notify: plugin_oep_notify,
    priority: 1,
    target_apps: &TARGET_APPS as *const [u16; 11] as *const u16,
};

extern "C" fn plugin_init(_v: Version) -> i32 {
    1
}

extern "C" fn plugin_oep_notify(_v: Version) {
    let _ = catch_unwind(|| {
        static START_ONCE: Once = Once::new();
        START_ONCE.call_once(|| {
            let _ = std::thread::Builder::new()
                .name("clientdbg".into())
                .spawn(|| {
                    let _ = catch_unwind(worker);
                });
        });
    });
}

// ---------------------------------------------------------------------------
// Shared state between the exception handler and the worker

/// One recorded breakpoint hit. Fixed size, no heap.
struct Hit {
    t_ms: u64,
    slot: u8,
    tid: u32,
    addr: u32,
    regs: RegSet,
    cap: CapBuf,
}

/// Configuration of one breakpoint slot. Replaced as a whole through an
/// atomic pointer. Old values are leaked on purpose, so the exception
/// handler never reads freed memory.
struct SlotCfg {
    addr: u32,
    kind: Kind,
    len: u8,
    items: Vec<Item>,
    limit: u32,
    hits: AtomicU32,
}

static RING: OnceLock<Ring<Hit, RING_SLOTS>> = OnceLock::new();
static START: OnceLock<Instant> = OnceLock::new();
static SLOTS: [AtomicPtr<SlotCfg>; 4] = [
    AtomicPtr::new(null_mut()),
    AtomicPtr::new(null_mut()),
    AtomicPtr::new(null_mut()),
    AtomicPtr::new(null_mut()),
];
static LAST_DROPPED: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    START
        .get()
        .map(|s| s.elapsed().as_millis() as u64)
        .unwrap_or(0)
}

unsafe extern "system" fn veh(ep: *mut EXCEPTION_POINTERS) -> i32 {
    if ep.is_null() {
        return CONTINUE_SEARCH;
    }
    let rec = (*ep).ExceptionRecord;
    let ctx = (*ep).ContextRecord;
    if rec.is_null() || ctx.is_null() || (*rec).ExceptionCode != EXCEPTION_SINGLE_STEP {
        return CONTINUE_SEARCH;
    }
    let ctx = &mut *ctx;
    let dr6 = ctx.Dr6;
    if dr6 & 0xf == 0 {
        return CONTINUE_SEARCH; // not a hardware breakpoint hit
    }
    let Some(ring) = RING.get() else {
        return CONTINUE_SEARCH;
    };
    let regs = RegSet {
        eax: ctx.Eax,
        ecx: ctx.Ecx,
        edx: ctx.Edx,
        ebx: ctx.Ebx,
        esp: ctx.Esp,
        ebp: ctx.Ebp,
        esi: ctx.Esi,
        edi: ctx.Edi,
        eip: ctx.Eip,
        eflags: ctx.EFlags,
    };
    for slot in 0..4u8 {
        if dr6 & (1 << slot) == 0 {
            continue;
        }
        let p = SLOTS[slot as usize].load(Ordering::Acquire);
        if p.is_null() {
            continue; // slot was cleared, thread has old registers until the next apply
        }
        let cfg = &*p;
        let n = cfg.hits.fetch_add(1, Ordering::Relaxed);
        if cfg.limit != 0 && n >= cfg.limit {
            continue;
        }
        let mut cap = CapBuf::default();
        capture::run(&cfg.items, &regs, &win::SelfMem, &mut cap);
        ring.push(Hit {
            t_ms: now_ms(),
            slot,
            tid: win::current_tid(),
            addr: cfg.addr,
            regs,
            cap,
        });
    }
    ctx.Dr6 = 0;
    ctx.EFlags |= RESUME_FLAG; // resume flag: do not hit the same instruction again
    CONTINUE_EXECUTION
}

fn install_veh() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = RING.get_or_init(Ring::new);
        let _ = START.get_or_init(Instant::now);
        unsafe { AddVectoredExceptionHandler(1, Some(veh)) };
    });
}

fn wanted_regs() -> win::DrSet {
    let mut w = win::DrSet::default();
    for (i, s) in SLOTS.iter().enumerate() {
        let p = s.load(Ordering::Acquire);
        if p.is_null() {
            continue;
        }
        // SAFETY: slot configs are never freed.
        let cfg = unsafe { &*p };
        w.addr[i] = cfg.addr;
        w.dr7 |= dr::dr7_bits(i as u8, cfg.kind, cfg.len);
    }
    w
}

fn hit_to_event(h: &Hit) -> Event {
    let caps = h
        .cap
        .entries()
        .map(|(e, bytes)| CapOut {
            item: e.item.to_string(),
            ptr: e.ptr,
            hex: hex::encode(bytes),
            err: match e.status {
                ST_OK => None,
                ST_TRUNCATED => Some("truncated".to_string()),
                _ => Some("unreadable".to_string()),
            },
        })
        .collect();
    Event::Hbp {
        t: h.t_ms,
        slot: h.slot,
        tid: h.tid,
        addr: h.addr,
        regs: h.regs,
        caps,
    }
}

// ---------------------------------------------------------------------------
// Worker

fn dll_dir() -> PathBuf {
    win::dll_path()
        .map(PathBuf::from)
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default()
}

fn worker() {
    let dir = dll_dir();
    let cfg = ini::parse(&std::fs::read_to_string(dir.join("clientdbg.ini")).unwrap_or_default());
    let token = std::fs::read_to_string(dir.join("clientdbg.token"))
        .unwrap_or_default()
        .trim()
        .to_string();
    if token.is_empty() {
        return; // no token: do nothing
    }
    install_veh();
    loop {
        let _ = catch_unwind(|| session(&cfg, &token));
        std::thread::sleep(Duration::from_millis(RECONNECT_MS));
    }
}

fn send_line(s: &mut TcpStream, line: &str) -> std::io::Result<()> {
    let mut v = Vec::with_capacity(line.len() + 1);
    v.extend_from_slice(line.as_bytes());
    v.push(b'\n');
    s.write_all(&v)
}

fn session(cfg: &ini::Config, token: &str) -> std::io::Result<()> {
    let addr = (cfg.host.as_str(), cfg.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no address"))?;
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(3))?;
    s.set_nodelay(true)?;
    s.set_read_timeout(Some(Duration::from_millis(50)))?;
    s.set_write_timeout(Some(Duration::from_secs(5)))?;

    let base = win::exe_base();
    let hello = HelloMsg {
        hello: Hello {
            pid: win::current_pid(),
            exe: win::exe_path(),
            base,
            size: win::exe_size(base),
            version: concat!("clientdbg ", env!("CARGO_PKG_VERSION")).to_string(),
        },
        token: token.to_string(),
    };
    send_line(&mut s, &proto::encode(&hello))?;

    let mut inbuf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 4096];
    let mut last_apply = Instant::now();
    loop {
        // Events first.
        if let Some(ring) = RING.get() {
            while let Some(h) = ring.pop() {
                send_line(&mut s, &proto::encode(&hit_to_event(&h)))?;
            }
            let d = ring.dropped();
            if d != LAST_DROPPED.swap(d, Ordering::Relaxed) {
                send_line(
                    &mut s,
                    &proto::encode(&Event::Stats {
                        t: now_ms(),
                        dropped: d,
                    }),
                )?;
            }
        }
        // Commands.
        match s.read(&mut tmp) {
            Ok(0) => return Ok(()),
            Ok(n) => inbuf.extend_from_slice(&tmp[..n]),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(e) => return Err(e),
        }
        if inbuf.len() > (1 << 20) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "line too long",
            ));
        }
        while let Some(pos) = inbuf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = inbuf.drain(..=pos).collect();
            let text = String::from_utf8_lossy(&line);
            let reply = handle_line(&text, cfg);
            send_line(&mut s, &reply)?;
        }
        // Periodic work.
        if last_apply.elapsed() >= Duration::from_millis(REAPPLY_MS) {
            last_apply = Instant::now();
            expire_limited(&mut s)?;
            let want = wanted_regs();
            if want.dr7 != 0 {
                win::apply_debug_regs(want); // new threads get the registers here
            }
        }
    }
}

/// Clear slots that reached their count limit.
fn expire_limited(s: &mut TcpStream) -> std::io::Result<()> {
    let mut any = false;
    for (i, slot) in SLOTS.iter().enumerate() {
        let p = slot.load(Ordering::Acquire);
        if p.is_null() {
            continue;
        }
        // SAFETY: slot configs are never freed.
        let cfg = unsafe { &*p };
        if cfg.limit != 0 && cfg.hits.load(Ordering::Relaxed) >= cfg.limit {
            slot.store(null_mut(), Ordering::Release);
            any = true;
            let ev = Event::Log {
                t: now_ms(),
                msg: format!("slot {i}: count limit {} reached, cleared", cfg.limit),
            };
            send_line(s, &proto::encode(&ev))?;
        }
    }
    if any {
        win::apply_debug_regs(wanted_regs());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Commands

fn handle_line(line: &str, cfg: &ini::Config) -> String {
    match proto::decode_request(line) {
        Ok(req) => {
            let id = req.id;
            let rep = catch_unwind(|| exec(req, cfg))
                .unwrap_or_else(|_| Reply::err(id, "internal error"));
            proto::encode(&rep)
        }
        Err(e) => {
            let id = serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .and_then(|v| v.get("id").and_then(|x| x.as_u64()))
                .unwrap_or(0);
            proto::encode(&Reply::err(id, e.to_string()))
        }
    }
}

fn exec(req: Request, cfg: &ini::Config) -> Reply {
    let id = req.id;
    match req.cmd {
        Command::Ping => Reply::ok(id, &"pong"),
        Command::Modules => match win::modules() {
            Ok(m) => Reply::ok(id, &m),
            Err(e) => Reply::err(id, e),
        },
        Command::Threads => Reply::ok(id, &win::threads()),
        Command::Rebase => Reply::ok(
            id,
            &RebaseInfo {
                base: win::exe_base(),
            },
        ),
        Command::Read { addr, len } => {
            if len > MAX_READ {
                return Reply::err(id, format!("len is more than {MAX_READ}"));
            }
            let mut buf = vec![0u8; len as usize];
            if win::read_mem(addr, &mut buf) {
                Reply::ok(id, &hex::encode(&buf))
            } else {
                Reply::err(id, format!("memory at {addr:#x} (+{len}) is not readable"))
            }
        }
        Command::Write { addr, hex: h } => {
            if !cfg.allow_write {
                return Reply::err(id, "write is off: set allow_write=1 in clientdbg.ini");
            }
            let data = match hex::decode(&h) {
                Ok(d) if d.len() <= MAX_WRITE => d,
                Ok(_) => return Reply::err(id, format!("write is more than {MAX_WRITE} bytes")),
                Err(e) => return Reply::err(id, e),
            };
            match win::write_mem(addr, &data) {
                Ok(()) => Reply::ok(id, &data.len()),
                Err(e) => Reply::err(id, e),
            }
        }
        Command::Regs { tid } => match win::regs(tid) {
            Ok(r) => Reply::ok(id, &r),
            Err(e) => Reply::err(id, e),
        },
        Command::Hbp {
            addr,
            kind,
            len,
            slot,
            capture: spec,
            limit,
        } => {
            if let Err(e) = dr::validate(addr, kind, len, slot) {
                return Reply::err(id, e);
            }
            let items = match capture::parse(&spec) {
                Ok(i) => i,
                Err(e) => return Reply::err(id, e.to_string()),
            };
            install_veh();
            let new = Box::new(SlotCfg {
                addr,
                kind,
                len,
                items,
                limit,
                hits: AtomicU32::new(0),
            });
            // The old value is leaked on purpose (see `SlotCfg`).
            SLOTS[slot as usize].store(Box::into_raw(new), Ordering::Release);
            let n = win::apply_debug_regs(wanted_regs());
            Reply::ok(id, &n)
        }
        Command::HbpClear { slot } => {
            match slot {
                Some(s) if (s as usize) < SLOTS.len() => {
                    SLOTS[s as usize].store(null_mut(), Ordering::Release)
                }
                Some(_) => return Reply::err(id, "slot must be 0 to 3"),
                None => SLOTS
                    .iter()
                    .for_each(|s| s.store(null_mut(), Ordering::Release)),
            }
            let n = win::apply_debug_regs(wanted_regs());
            Reply::ok(id, &n)
        }
    }
}
