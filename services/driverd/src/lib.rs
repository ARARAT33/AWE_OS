#![no_std]
#![allow(dead_code)]
#![allow(unused_attributes)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::module_inception)]

//! AWE Driver Microkernel (driverd): isolated, restartable hardware plane.
//! CellKernel owns only capabilities/IPC; concrete hardware discovery and
//! driver execution stay inside this separate service plane.

mod access;
mod acpi;
mod acpi_root;
mod apic;
mod asd;
mod catalog;
mod contract;
mod dependency;
mod pci;
mod pci_x86;
mod protocol;
mod reference;
mod registry;
mod supervisor;
mod virtio;

pub use access::{
    power_transition_allowed, AccessKind, AccessRegion, HardwareAccessPlan, InterruptMode,
    InterruptOwnership, PowerState,
};
pub use acpi::{
    checksum_ok, find_table, parse_header, parse_madt, AcpiError, AcpiTableRef, MadtRecord,
    SdtHeader,
};
pub use acpi_root::{parse_pointer_table, validate_rsdp, RsdpError, RsdpInfo};
pub use apic::{ApicError, IoApic, IrqRoute, LocalApic};
pub use asd::{
    package_transition, validate_asd, AsdError, AsdHeader, PackageState, ASD_HEADER_LEN,
    ASD_MAGIC, ASD_MAX_MANIFEST, ASD_MAX_PAYLOAD, ASD_MIN_SIGNATURE, ASD_VERSION,
};
pub use catalog::{descriptors, BuiltinDriver, BUILTIN_DRIVER_COUNT, BUILTIN_DRIVERS};
pub use contract::{
    lifecycle_maps_to_state, transition, DriverLifecycle, DriverManifest, DriverTrust,
    LifecycleError,
};
pub use dependency::{
    DependencyError, DependencyGraph, DriverDependency, DriverHealth, ResourceOwnership,
};
pub use pci::{
    enumerate as enumerate_pci, PciConfigAccess, PciDevice, PciDeviceTable, PciError, PciLocation,
};
#[cfg(target_arch = "x86_64")]
pub use pci_x86::X86CfgIo;
pub use protocol::{DriverClass, DriverCommand, DriverEvent, DriverId, DriverReply, DriverState};
pub use reference::{
    validate_block_request, validate_display_rect, BlockRequest, DisplayRect, InputEvent, NetFrame,
    ReferenceDevice, ReferenceError,
};
pub use registry::{DriverDescriptor, DriverRegistry, RegistryError};
pub use supervisor::{DriverSupervisor, SupervisorError};
pub use virtio::{VIRTIO_F_VERSION_1, VirtioDevice, VirtioError, VirtioKind, VirtioQueue};

/// Driver service ABI frozen by the AWE_OS 60.2 contract.
pub const DRIVERD_ABI_MAJOR: u16 = 1;
pub const DRIVERD_ABI_MINOR: u16 = 2;
pub const MAX_REGISTERED_DRIVERS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverdInfo {
    pub abi_major: u16,
    pub abi_minor: u16,
    pub max_drivers: usize,
    pub builtin_drivers: usize,
}

impl DriverdInfo {
    pub const fn current() -> Self {
        Self {
            abi_major: DRIVERD_ABI_MAJOR,
            abi_minor: DRIVERD_ABI_MINOR,
            max_drivers: MAX_REGISTERED_DRIVERS,
            builtin_drivers: BUILTIN_DRIVER_COUNT,
        }
    }
}
