//! Win32 layer for the 32-bit client. Only built for 32-bit Windows.
//!
//! Rules for code that runs while another thread is suspended: no heap
//! allocation and no lock between `SuspendThread` and `ResumeThread`.
//! The suspended thread can hold the heap lock or the loader lock.

use crate::capture::MemRead;
use crate::proto::{ModuleInfo, RegDump, RegSet, ThreadInfo};
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use windows_sys::Wdk::System::Threading::NtQueryInformationThread;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::Diagnostics::Debug::{
    GetThreadContext, ReadProcessMemory, SetThreadContext, WriteProcessMemory, CONTEXT,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Thread32First, Thread32Next,
    MODULEENTRY32W, THREADENTRY32,
};
use windows_sys::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleExW, GetModuleHandleW,
};
use windows_sys::Win32::System::Memory::{VirtualProtect, VirtualQuery, MEMORY_BASIC_INFORMATION};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId, OpenThread, ResumeThread,
    SuspendThread,
};

const THREAD_SUSPEND_RESUME: u32 = 0x2;
const THREAD_GET_CONTEXT: u32 = 0x8;
const THREAD_SET_CONTEXT: u32 = 0x10;
const THREAD_QUERY_INFORMATION: u32 = 0x40;
const TH32CS_SNAPTHREAD: u32 = 0x4;
const TH32CS_SNAPMODULE: u32 = 0x8;
const TH32CS_SNAPMODULE32: u32 = 0x10;
const CONTEXT_I386: u32 = 0x0001_0000;
const CONTEXT_CONTROL: u32 = CONTEXT_I386 | 0x1;
const CONTEXT_INTEGER: u32 = CONTEXT_I386 | 0x2;
const CONTEXT_SEGMENTS: u32 = CONTEXT_I386 | 0x4;
const CONTEXT_DEBUG_REGISTERS: u32 = CONTEXT_I386 | 0x10;
const MEM_COMMIT: u32 = 0x1000;
const PAGE_NOACCESS: u32 = 0x01;
const PAGE_GUARD: u32 = 0x100;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;
const READABLE: u32 = 0x02 | 0x04 | 0x08 | 0x20 | 0x40 | 0x80;
const GET_MODULE_HANDLE_EX_FROM_ADDRESS: u32 = 0x4;
const GET_MODULE_HANDLE_EX_UNCHANGED_REFCOUNT: u32 = 0x2;

pub fn current_tid() -> u32 {
    unsafe { GetCurrentThreadId() }
}

pub fn current_pid() -> u32 {
    unsafe { GetCurrentProcessId() }
}

fn wstr(buf: &[u16]) -> String {
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..n])
}

/// Image base of `Client.exe`.
pub fn exe_base() -> u32 {
    unsafe { GetModuleHandleW(null()) as usize as u32 }
}

pub fn exe_path() -> String {
    let mut buf = [0u16; 520];
    let n = unsafe { GetModuleFileNameW(null_mut(), buf.as_mut_ptr(), buf.len() as u32) } as usize;
    String::from_utf16_lossy(&buf[..n.min(buf.len())])
}

/// Path of this DLL.
pub fn dll_path() -> Option<String> {
    let mut h: windows_sys::Win32::Foundation::HMODULE = null_mut();
    let here = dll_path as *const () as *const u16;
    let ok = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FROM_ADDRESS | GET_MODULE_HANDLE_EX_UNCHANGED_REFCOUNT,
            here,
            &mut h,
        )
    };
    if ok == 0 {
        return None;
    }
    let mut buf = [0u16; 520];
    let n = unsafe { GetModuleFileNameW(h, buf.as_mut_ptr(), buf.len() as u32) } as usize;
    (n > 0).then(|| String::from_utf16_lossy(&buf[..n.min(buf.len())]))
}

/// `SizeOfImage` from the PE header of the exe.
pub fn exe_size(base: u32) -> u32 {
    let mut lfanew = [0u8; 4];
    if !read_mem(base.wrapping_add(0x3c), &mut lfanew) {
        return 0;
    }
    let nt = base.wrapping_add(u32::from_le_bytes(lfanew));
    let mut size = [0u8; 4];
    if !read_mem(nt.wrapping_add(0x50), &mut size) {
        return 0;
    }
    u32::from_le_bytes(size)
}

/// Check that `[addr, addr+len)` is committed, readable and not a guard page.
fn range_readable(addr: u32, len: usize) -> bool {
    let end = addr as u64 + len as u64;
    if end > 0x1_0000_0000 {
        return false;
    }
    let mut cur = addr as u64;
    while cur < end {
        let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        let got = unsafe {
            VirtualQuery(
                cur as usize as *const c_void,
                &mut mbi,
                std::mem::size_of_val(&mbi),
            )
        };
        if got == 0 || mbi.State != MEM_COMMIT {
            return false;
        }
        if mbi.Protect & (PAGE_NOACCESS | PAGE_GUARD) != 0 || mbi.Protect & READABLE == 0 {
            return false;
        }
        cur = mbi.BaseAddress as usize as u64 + mbi.RegionSize as u64;
    }
    true
}

/// Guarded read from this process. `ReadProcessMemory` returns an error
/// instead of raising an access violation if the page goes away.
pub fn read_mem(addr: u32, out: &mut [u8]) -> bool {
    if out.is_empty() {
        return true;
    }
    if !range_readable(addr, out.len()) {
        return false;
    }
    let mut got = 0usize;
    let ok = unsafe {
        ReadProcessMemory(
            GetCurrentProcess(),
            addr as usize as *const c_void,
            out.as_mut_ptr() as *mut c_void,
            out.len(),
            &mut got,
        )
    };
    ok != 0 && got == out.len()
}

/// Write with `VirtualProtect`; the old protection is restored.
pub fn write_mem(addr: u32, data: &[u8]) -> Result<(), String> {
    if data.is_empty() {
        return Ok(());
    }
    let p = addr as usize as *const c_void;
    let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { VirtualQuery(p, &mut mbi, std::mem::size_of_val(&mbi)) } == 0
        || mbi.State != MEM_COMMIT
    {
        return Err("address is not committed".into());
    }
    let mut old = 0u32;
    if unsafe { VirtualProtect(p, data.len(), PAGE_EXECUTE_READWRITE, &mut old) } == 0 {
        return Err("VirtualProtect failed".into());
    }
    let mut done = 0usize;
    let ok = unsafe {
        WriteProcessMemory(
            GetCurrentProcess(),
            p,
            data.as_ptr() as *const c_void,
            data.len(),
            &mut done,
        )
    };
    let mut tmp = 0u32;
    unsafe { VirtualProtect(p, data.len(), old, &mut tmp) };
    if ok == 0 || done != data.len() {
        return Err("WriteProcessMemory failed".into());
    }
    Ok(())
}

/// `MemRead` for capture items.
pub struct SelfMem;

impl MemRead for SelfMem {
    fn read(&self, addr: u32, out: &mut [u8]) -> bool {
        read_mem(addr, out)
    }
}

pub fn modules() -> Result<Vec<ModuleInfo>, String> {
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, 0) };
    if snap as isize == -1 {
        return Err("CreateToolhelp32Snapshot failed".into());
    }
    let mut out = Vec::new();
    let mut e: MODULEENTRY32W = unsafe { std::mem::zeroed() };
    e.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;
    let mut ok = unsafe { Module32FirstW(snap, &mut e) };
    while ok != 0 {
        out.push(ModuleInfo {
            name: wstr(&e.szModule),
            base: e.modBaseAddr as usize as u32,
            size: e.modBaseSize,
        });
        ok = unsafe { Module32NextW(snap, &mut e) };
    }
    unsafe { CloseHandle(snap) };
    Ok(out)
}

/// Thread ids of this process.
pub fn thread_ids() -> Vec<u32> {
    let mut out = Vec::new();
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snap as isize == -1 {
        return out;
    }
    let pid = current_pid();
    let mut e: THREADENTRY32 = unsafe { std::mem::zeroed() };
    e.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
    let mut ok = unsafe { Thread32First(snap, &mut e) };
    while ok != 0 {
        if e.th32OwnerProcessID == pid {
            out.push(e.th32ThreadID);
        }
        e.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
        ok = unsafe { Thread32Next(snap, &mut e) };
    }
    unsafe { CloseHandle(snap) };
    out
}

fn zero_context() -> CONTEXT {
    unsafe { std::mem::zeroed() }
}

fn open_thread(tid: u32, access: u32) -> Option<HANDLE> {
    let h = unsafe { OpenThread(access, 0, tid) };
    (!h.is_null()).then_some(h)
}

fn thread_start(h: HANDLE) -> u32 {
    let mut start = 0u32;
    let mut ret = 0u32;
    // ThreadQuerySetWin32StartAddress = 9
    let st = unsafe {
        NtQueryInformationThread(h, 9, &mut start as *mut u32 as *mut c_void, 4, &mut ret)
    };
    if st >= 0 {
        start
    } else {
        0
    }
}

pub fn threads() -> Vec<ThreadInfo> {
    let own = current_tid();
    let tids = thread_ids();
    let mut out = Vec::with_capacity(tids.len());
    for tid in tids {
        let Some(h) = open_thread(
            tid,
            THREAD_QUERY_INFORMATION | THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT,
        ) else {
            out.push(ThreadInfo {
                tid,
                start: 0,
                eip: None,
            });
            continue;
        };
        let start = thread_start(h);
        let mut eip = None;
        if tid != own {
            let mut ctx = zero_context();
            ctx.ContextFlags = CONTEXT_CONTROL;
            // No allocation between suspend and resume.
            let ok = unsafe {
                if SuspendThread(h) == u32::MAX {
                    false
                } else {
                    let r = GetThreadContext(h, &mut ctx) != 0;
                    ResumeThread(h);
                    r
                }
            };
            if ok {
                eip = Some(ctx.Eip);
            }
        }
        unsafe { CloseHandle(h) };
        out.push(ThreadInfo { tid, start, eip });
    }
    out
}

pub fn regs(tid: u32) -> Result<RegDump, String> {
    if tid == current_tid() {
        return Err("cannot read the context of the plugin thread".into());
    }
    let h =
        open_thread(tid, THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT).ok_or("OpenThread failed")?;
    let mut ctx = zero_context();
    ctx.ContextFlags =
        CONTEXT_CONTROL | CONTEXT_INTEGER | CONTEXT_SEGMENTS | CONTEXT_DEBUG_REGISTERS;
    let ok = unsafe {
        if SuspendThread(h) == u32::MAX {
            false
        } else {
            let r = GetThreadContext(h, &mut ctx) != 0;
            ResumeThread(h);
            r
        }
    };
    unsafe { CloseHandle(h) };
    if !ok {
        return Err("GetThreadContext failed".into());
    }
    Ok(RegDump {
        regs: RegSet {
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
        },
        dr0: ctx.Dr0,
        dr1: ctx.Dr1,
        dr2: ctx.Dr2,
        dr3: ctx.Dr3,
        dr6: ctx.Dr6,
        dr7: ctx.Dr7,
    })
}

/// Wanted debug register values.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct DrSet {
    pub addr: [u32; 4],
    pub dr7: u32,
}

/// Set the debug registers of every thread except the caller.
/// Returns the number of threads that were changed.
pub fn apply_debug_regs(want: DrSet) -> usize {
    let own = current_tid();
    let tids = thread_ids(); // allocates, before any thread is suspended
    let mut changed = 0;
    for tid in tids {
        if tid == own {
            continue;
        }
        let Some(h) = open_thread(
            tid,
            THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_SET_CONTEXT,
        ) else {
            continue;
        };
        let mut ctx = zero_context();
        ctx.ContextFlags = CONTEXT_DEBUG_REGISTERS;
        // No allocation, no lock, no logging between suspend and resume.
        let did = unsafe {
            if SuspendThread(h) == u32::MAX {
                false
            } else {
                let mut did = false;
                if GetThreadContext(h, &mut ctx) != 0 {
                    let same = ctx.Dr0 == want.addr[0]
                        && ctx.Dr1 == want.addr[1]
                        && ctx.Dr2 == want.addr[2]
                        && ctx.Dr3 == want.addr[3]
                        && ctx.Dr7 == want.dr7;
                    if !same {
                        ctx.Dr0 = want.addr[0];
                        ctx.Dr1 = want.addr[1];
                        ctx.Dr2 = want.addr[2];
                        ctx.Dr3 = want.addr[3];
                        ctx.Dr6 = 0;
                        ctx.Dr7 = want.dr7;
                        did = SetThreadContext(h, &ctx) != 0;
                    }
                }
                ResumeThread(h);
                did
            }
        };
        unsafe { CloseHandle(h) };
        if did {
            changed += 1;
        }
    }
    changed
}
