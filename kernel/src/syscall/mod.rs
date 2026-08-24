#![no_std]

pub mod abi;
pub mod dispatch;

pub use abi::{
    ERR_BUSY, ERR_INVALID_ARGUMENT, ERR_NO_MEMORY, ERR_NOT_FOUND, ERR_OK, ERR_PERMISSION, Syscall,
    SyscallResult,
};
pub use dispatch::SyscallContext;

pub const IA32_EFER_MSR: u32 = 0xC000_0080;
pub const IA32_STAR_MSR: u32 = 0xC000_0081;
pub const IA32_LSTAR_MSR: u32 = 0xC000_0082;
pub const IA32_FMASK_MSR: u32 = 0xC000_0084;

/// Configures x86_64 Model Specific Registers for SYSCALL/SYSRET fast system call entry.
pub unsafe fn init_msr_syscall(syscall_handler_address: u64, kernel_cs: u16, user_cs: u16) {
    let _ = (syscall_handler_address, kernel_cs, user_cs);
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    unsafe {
        use crate::arch::x86_64::{read_msr, write_msr};
        // Enable SCE (System Call Extension) in EFER
        let efer = read_msr(IA32_EFER_MSR);
        write_msr(IA32_EFER_MSR, efer | 1);

        // STAR: Syscall CS/SS (bits 47..32) and Sysret CS/SS (bits 63..48)
        let star = ((kernel_cs as u64) << 32) | (((user_cs as u64) | 3) << 48);
        write_msr(IA32_STAR_MSR, star);

        // LSTAR: 64-bit SYSCALL target RIP
        write_msr(IA32_LSTAR_MSR, syscall_handler_address);

        // FMASK: Mask RFLAGS bits (e.g. disable interrupts IF bit 0x200 during syscall)
        write_msr(IA32_FMASK_MSR, 0x200);
    }
}
