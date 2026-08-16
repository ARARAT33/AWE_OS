#![no_std]

pub mod bus;
pub mod compat;
pub mod contract;
pub mod virtio;

pub use bus::{DeviceId, DeviceKind, DriverBus};
pub use compat::{bind_compatible_driver, validate_contract, CompatibilityRegistry, DriverManifest, DriverSource};
pub use contract::{DeviceContract, DmaPolicy, InterruptMode, MmioRegion};
pub use virtio::{VirtioDevice, VirtioFeatures};
