#![no_std]

pub mod abi;
pub mod dispatch;

pub use abi::{
    ERR_BUSY, ERR_INVALID_ARGUMENT, ERR_NO_MEMORY, ERR_NOT_FOUND, ERR_OK, ERR_PERMISSION, Syscall,
    SyscallResult,
};
pub use dispatch::SyscallContext;
