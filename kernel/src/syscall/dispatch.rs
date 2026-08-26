#![no_std]

use super::{Syscall, SyscallContext, SyscallResult};

pub const ERR_OK: u64 = 0;
pub const ERR_INVALID_ARGUMENT: u64 = 1;
pub const ERR_PERMISSION: u64 = 2;
pub const MAX_USER_COPY: usize = 4096;

impl<'a> SyscallContext<'a> {
    pub fn dispatch(&mut self, number: u64, args: [u64; 6]) -> SyscallResult {
        let syscall = match Syscall::try_from(number) {
            Ok(value) => value,
            Err(_) => return err(ERR_INVALID_ARGUMENT),
        };

        match syscall {
            Syscall::Exit => ok(0),
            Syscall::Yield => ok(0),
            Syscall::Sleep => ok(args[0]),
            Syscall::Map => {
                if !valid_user_page_range(args[0], args[1]) {
                    return err(ERR_INVALID_ARGUMENT);
                }
                ok(args[0])
            }
            Syscall::Unmap => {
                if !valid_user_page_range(args[0], args[1]) {
                    return err(ERR_INVALID_ARGUMENT);
                }
                ok(args[0])
            }
            Syscall::Read => {
                let len = core::cmp::min(args[1] as usize, MAX_USER_COPY);
                if !valid_user_range(args[0], len) {
                    return err(ERR_INVALID_ARGUMENT);
                }
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
                if !valid_user_range(args[0], len) {
                    return err(ERR_INVALID_ARGUMENT);
                }
                #[cfg(all(target_arch = "x86_64", target_os = "none"))]
                {
                    let ptr = args[0] as *const u8;
                    unsafe {
                        for i in 0..len {
                            let byte = core::ptr::read_volatile(ptr.add(i));
                            crate::arch::x86_64::serial_write_byte(byte);
                        }
                    }
                }
                ok(len as u64)
            }
        }
    }
}

const fn ok(value: u64) -> SyscallResult { SyscallResult { value, error: ERR_OK } }
const fn err(error: u64) -> SyscallResult { SyscallResult { value: 0, error } }

fn valid_user_page_range(address: u64, pages: u64) -> bool {
    pages != 0 && address % 4096 == 0 && pages <= 1024 && address.checked_add(pages * 4096).is_some()
}

fn valid_user_range(address: u64, len: usize) -> bool {
    len <= MAX_USER_COPY && address >= 0x1000 && address.checked_add(len as u64).is_some() && address.checked_add(len as u64).unwrap() <= 0x0000_8000_0000_0000
}
