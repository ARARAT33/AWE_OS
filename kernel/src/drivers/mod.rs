#![no_std]

pub mod bus;
pub mod compat;
pub mod contract;
pub mod learning;
pub mod virtio;
pub mod universal;
pub mod installer;

pub use bus::{DeviceId, DeviceKind, DriverBus};
pub use compat::{bind_compatible_driver, validate_contract, CompatibilityRegistry, DriverManifest, DriverSource};
pub use contract::{DeviceContract, DmaPolicy, InterruptMode, MmioRegion};
pub use learning::{DriverExperience, ExperienceDb, ProbeOutcome};
pub use virtio::{VirtioDevice, VirtioFeatures};
pub use universal::{validate_request, DriverAbi, DriverAction, DriverError, DriverOs, DriverRequest, DriverResult};
pub use installer::{plan_install, InstallError, InstallPlan, InstallerPackage, PackageFormat};
