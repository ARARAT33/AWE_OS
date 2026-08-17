#![no_std]

// CellKernel is intentionally hardware-driver free.
// All hardware discovery, driver lifecycle, compatibility adapters and
// VirtIO/Linux/Windows/Android driver execution live in services/driverd.
// The kernel owns only the minimal IPC/capability boundary needed to talk to
// that isolated service.
pub mod arch;
pub mod boot_phase;
pub mod boot_state;
pub mod device;
pub mod entry;
pub mod formats;
pub mod interrupts;
pub mod ipc;
pub mod logging;
pub mod memory;
pub mod net;
pub mod platform;
pub mod process;
pub mod runtime;
pub mod scheduler;
pub mod security;
pub mod service;
pub mod storage;
pub mod system_contract;
pub mod syscall;
pub mod time;
