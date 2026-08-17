#![no_std]

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub mod riscv;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;
