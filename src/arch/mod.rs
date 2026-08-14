//! Architecture abstraction for AWEOS.
//! Each backend owns the tiny set of privileged operations required by the kernel.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;
#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub mod riscv;

pub trait Architecture {
    /// Halt the current CPU until the next interrupt/event.
    fn halt() -> !;
    /// Disable interrupts and return the previous interrupt state.
    fn irq_save() -> usize;
    /// Restore a previously saved interrupt state.
    unsafe fn irq_restore(state: usize);
}
