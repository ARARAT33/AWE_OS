#![no_std]

//! AWE Driver Microkernel (driverd).
//!
//! This crate is the isolated hardware-services plane of AWE_OS. CellKernel
//! does not link driver implementations. Instead it exposes a tiny IPC and
//! capability boundary to driverd, allowing a faulty or incompatible driver
//! to be restarted/quarantined without taking down the kernel.

mod protocol;
mod registry;
mod supervisor;

pub use protocol::{DriverClass, DriverCommand, DriverEvent, DriverId, DriverReply, DriverState};
pub use registry::{DriverDescriptor, DriverRegistry, RegistryError};
pub use supervisor::{DriverSupervisor, SupervisorError};

/// ABI version shared by CellKernel and driverd.
pub const DRIVERD_ABI_MAJOR: u16 = 1;
pub const DRIVERD_ABI_MINOR: u16 = 0;

/// Hard upper bound used by the bootstrap supervisor. A production build can
/// negotiate a larger value after memory and IPC quotas are established.
pub const MAX_REGISTERED_DRIVERS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverdInfo {
    pub abi_major: u16,
    pub abi_minor: u16,
    pub max_drivers: usize,
}

impl DriverdInfo {
    pub const fn current() -> Self {
        Self {
            abi_major: DRIVERD_ABI_MAJOR,
            abi_minor: DRIVERD_ABI_MINOR,
            max_drivers: MAX_REGISTERED_DRIVERS,
        }
    }
}
