//! clientdbg: live debugger plugin for `Client.exe` 381 and its host program.
//!
//! Modules that need no Windows are plain and tested on Linux. The plugin
//! (`plugin`, `win`) is built only for 32-bit Windows.

pub mod addr;
pub mod capture;
pub mod cli;
pub mod dr;
pub mod hex;
pub mod host;
pub mod ini;
pub mod proto;
pub mod ring;

#[cfg(all(windows, target_arch = "x86"))]
pub mod plugin;
#[cfg(all(windows, target_arch = "x86"))]
pub mod win;
