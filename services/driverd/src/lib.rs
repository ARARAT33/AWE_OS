#![no_std]

//! AWE Driver Microkernel (driverd): isolated, restartable hardware plane.
//! CellKernel owns only capabilities/IPC; concrete hardware discovery and
//! driver execution stay inside this separate service plane.

mod access;
mod acpi;
mod acpi_root;
mod apic;
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

pub use access::{AccessKind, AccessRegion, HardwareAccessPlan, InterruptMode, InterruptOwnership, PowerState, power_transition_allowed};
pub use acpi::{AcpiError, AcpiTableRef, MadtRecord, SdtHeader, checksum_ok, find_table, parse_header, parse_madt};
pub use acpi_root::{RsdpError, RsdpInfo, parse_pointer_table, validate_rsdp};
pub use apic::{ApicError, IoApic, IrqRoute, LocalApic};
pub use catalog::{BuiltinDriver, BUILTIN_DRIVER_COUNT, BUILTIN_DRIVERS, descriptors};
pub use contract::{DriverLifecycle, DriverManifest, DriverTrust, LifecycleError, lifecycle_maps_to_state, transition};
pub use dependency::{DependencyError, DependencyGraph, DriverHealth, DriverDependency, ResourceOwnership};
pub use pci::{PciConfigAccess, PciDevice, PciDeviceTable, PciError, PciLocation, enumerate as enumerate_pci};
#[cfg(target_arch = "x86_64")]
pub use pci_x86::X86CfgIo;
pub use protocol::{DriverClass, DriverCommand, DriverEvent, DriverId, DriverReply, DriverState};
pub use reference::{BlockRequest, DisplayRect, InputEvent, NetFrame, ReferenceDevice, ReferenceError, validate_block_request, validate_display_rect};
pub use registry::{DriverDescriptor, DriverRegistry, RegistryError};
pub use supervisor::{DriverSupervisor, SupervisorError};
pub use virtio::{VirtioDevice, VirtioError, VirtioKind, VirtioQueue, VIRTIO_F_VERSION_1};

/// Driver service ABI frozen by the AWE_OS 60.2 contract.
pub const DRIVERD_ABI_MAJOR: u16 = 1;
pub const DRIVERD_ABI_MINOR: u16 = 2;
pub const MAX_REGISTERED_DRIVERS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverdInfo { pub abi_major: u16, pub abi_minor: u16, pub max_drivers: usize, pub builtin_drivers: usize }

impl DriverdInfo {
    pub const fn current() -> Self {
        Self { abi_major: DRIVERD_ABI_MAJOR, abi_minor: DRIVERD_ABI_MINOR, max_drivers: MAX_REGISTERED_DRIVERS, builtin_drivers: BUILTIN_DRIVER_COUNT }
    }
}
