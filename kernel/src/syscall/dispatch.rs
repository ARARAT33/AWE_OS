#![no_std]

use super::abi::{ERR_INVALID_ARGUMENT, ERR_OK, ERR_PERMISSION, Syscall, SyscallResult};
use crate::process::{ProcessDescriptor, ProcessState};

const MAX_USER_COPY: usize = 4096;
const USER_TOP: u64 = 0x0000_8000_0000_0000;
const PAGE_SIZE: u64 = 4096;

fn valid_user_range(ptr: u64, len: usize) -> bool {
    if ptr == 0 || len == 0 { return false; }
    let len_u64 = match u64::try_from(len) { Ok(v) => v, Err(_) => return false };
    let end = match ptr.checked_add(len_u64 - 1) { Some(v) => v, None => return false };
    ptr < USER_TOP && end < USER_TOP
}

fn valid_user_page_range(addr: u64, size: u64) -> bool {
    if size == 0 || addr % PAGE_SIZE != 0 { return false; }
    let end = match addr.checked_add(size - 1) { Some(v) => v, None => return false };
    addr < USER_TOP && end < USER_TOP
}

pub struct SyscallContext<'a> { pub process: &'a mut ProcessDescriptor }

impl<'a> SyscallContext<'a> {
    pub fn dispatch(&mut self, number: u64, args: [u64; 6]) -> SyscallResult {
        let call = match number {
            0 => Syscall::Yield,
            1 => Syscall::Exit,
            2 => Syscall::Spawn,
            3 => Syscall::IpcSend,
            4 => Syscall::IpcRecv,
            5 => Syscall::Map,
            6 => Syscall::Unmap,
            7 => Syscall::Read,
            8 => Syscall::Write,
            _ => return err(ERR_INVALID_ARGUMENT),
        };
        match call {
            Syscall::Yield => {
                if self.process.state != ProcessState::Running { return err(ERR_PERMISSION); }
                self.process.state = ProcessState::Runnable;
                ok(0)
            }
            Syscall::Exit => {
                if self.process.state == ProcessState::Exited { return err(ERR_PERMISSION); }
                self.process.state = ProcessState::Exited;
                ok(0)
            }
            Syscall::Spawn => err(ERR_PERMISSION),
            Syscall::IpcSend | Syscall::IpcRecv => {
                if args[1] == 0 || args[1] > 4 { return err(ERR_INVALID_ARGUMENT); }
                if !self.process.budget.consume_ipc(1) { return err(ERR_PERMISSION); }
                ok(args[0])
            }
            Syscall::Map => {
                if !valid_user_page_range(args[0], args[1]) { return err(ERR_INVALID_ARGUMENT); }
                if !self.process.budget.consume_memory(args[1]) { return err(ERR_PERMISSION); }
                ok(args[0])
            }
            Syscall::Unmap => {
                if !valid_user_page_range(args[0], args[1]) { return err(ERR_INVALID_ARGUMENT); }
                ok(args[0])
            }
            Syscall::Read => {
                let len = core::cmp::min(args[1] as usize, MAX_USER_COPY);
                if !valid_user_range(args[0], len) { return err(ERR_INVALID_ARGUMENT); }
                let initrd = b"INITRD_SHELL_IMAGE_OK";
                let copy_len = core::cmp::min(len, initrd.len());
                let ptr = args[0] as *mut u8;
                unsafe {
                    for (i, byte) in initrd[..copy_len].iter().enumerate() {
                        core::ptr::write_volatile(ptr.add(i), *byte);
                    }
                }
                ok(copy_len as u64)
            }
            Syscall::Write => {
                let len = core::cmp::min(args[1] as usize, MAX_USER_COPY);
                if !valid_user_range(args[0], len) { return err(ERR_INVALID_ARGUMENT); }
                let ptr = args[0] as *const u8;
                #[cfg(all(target_arch = "x86_64", target_os = "none"))]
                unsafe {
                    for i in 0..len {
                        let byte = core::ptr::read_volatile(ptr.add(i));
                        crate::arch::x86_64::serial_write_byte(byte);
                    }
                }
                ok(len as u64)
            }
        }
    }
}

const fn ok(value: u64) -> SyscallResult { SyscallResult { value, error: ERR_OK } }
const fn err(error: u64) -> SyscallResult { SyscallResult { value: 0, error } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{ProcessId, ResourceBudget};
    fn descriptor() -> ProcessDescriptor {
        ProcessDescriptor { id: ProcessId(1), state: ProcessState::Running, budget: ResourceBudget { cpu_ticks: 10, memory_bytes: 8192, ipc_messages: 2 } }
    }
    #[test] fn invalid_call_is_rejected() { let mut p = descriptor(); let mut c = SyscallContext { process: &mut p }; assert_eq!(c.dispatch(99, [0;6]).error, ERR_INVALID_ARGUMENT); }
    #[test] fn yield_and_exit_change_state() { let mut p = descriptor(); let mut c = SyscallContext { process: &mut p }; assert_eq!(c.dispatch(Syscall::Yield as u64, [0;6]).error, ERR_OK); assert_eq!(c.process.state, ProcessState::Runnable); c.process.state = ProcessState::Running; assert_eq!(c.dispatch(Syscall::Exit as u64, [0;6]).error, ERR_OK); assert_eq!(c.process.state, ProcessState::Exited); }
    #[test] fn pointer_validation_is_fail_closed() {
        let mut p = descriptor(); let mut c = SyscallContext { process: &mut p }; let buf = [0u8;8];
        assert_eq!(c.dispatch(Syscall::Read as u64, [buf.as_ptr() as u64, 0,0,0,0,0]).error, ERR_INVALID_ARGUMENT);
        assert_eq!(c.dispatch(Syscall::Read as u64, [USER_TOP - 4, 8,0,0,0,0]).error, ERR_INVALID_ARGUMENT);
        assert_eq!(c.dispatch(Syscall::Write as u64, [buf.as_ptr() as u64, 8,0,0,0,0]).error, ERR_OK);
    }
    #[test] fn memory_mapping_requires_page_alignment_and_budget() {
        let mut p = descriptor(); let mut c = SyscallContext { process: &mut p };
        assert_eq!(c.dispatch(Syscall::Map as u64, [0x4000, 4096,0,0,0,0]).error, ERR_OK);
        assert_eq!(c.dispatch(Syscall::Map as u64, [0x4001, 4096,0,0,0,0]).error, ERR_INVALID_ARGUMENT);
        assert_eq!(c.dispatch(Syscall::Map as u64, [0x8000, 4096,0,0,0,0]).error, ERR_PERMISSION);
    }
    #[test] fn ipc_budget_is_enforced() {
        let mut p = descriptor(); let mut c = SyscallContext { process: &mut p };
        assert_eq!(c.dispatch(Syscall::IpcSend as u64, [7,1,0,0,0,0]).error, ERR_OK);
        assert_eq!(c.dispatch(Syscall::IpcSend as u64, [8,1,0,0,0,0]).error, ERR_OK);
        assert_eq!(c.dispatch(Syscall::IpcSend as u64, [9,1,0,0,0,0]).error, ERR_PERMISSION);
    }
    #[test] fn read_returns_bounded_initrd_payload() {
        let mut p = descriptor(); let mut c = SyscallContext { process: &mut p }; let mut buf = [0u8;32];
        let result = c.dispatch(Syscall::Read as u64, [buf.as_mut_ptr() as u64,32,0,0,0,0]);
        assert_eq!(result.error, ERR_OK); assert_eq!(&buf[..result.value as usize], b"INITRD_SHELL_IMAGE_OK");
    }
}