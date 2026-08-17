#![no_std]

//! AWE Driver Microkernel (driverd): isolated, restartable hardware plane.
//! CellKernel owns only capabilities/IPC; driver implementations never link
//! into the kernel image.

mod catalog;
mod protocol;
mod registry;
mod supervisor;

pub use catalog::{BuiltinDriver, BUILTIN_DRIVER_COUNT, BUILTIN_DRIVERS, descriptors};
pub use protocol::{DriverClass, DriverCommand, DriverEvent, DriverId, DriverReply, DriverState};
pub use registry::{DriverDescriptor, DriverRegistry, RegistryError};
pub use supervisor::{DriverSupervisor, SupervisorError};

pub const DRIVERD_ABI_MAJOR: u16 = 1;
pub const DRIVERD_ABI_MINOR: u16 = 0;
pub const MAX_REGISTERED_DRIVERS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverdInfo { pub abi_major: u16, pub abi_minor: u16, pub max_drivers: usize, pub builtin_drivers: usize }

impl DriverdInfo {
    pub const fn current() -> Self {
        Self { abi_major: DRIVERD_ABI_MAJOR, abi_minor: DRIVERD_ABI_MINOR, max_drivers: MAX_REGISTERED_DRIVERS, builtin_drivers: BUILTIN_DRIVER_COUNT }
    }
}
