#![no_std]

pub mod android;
pub mod bus;
pub mod compat;
pub mod contract;
pub mod core;
pub mod learning;
pub mod linux;
pub mod linux_install;
pub mod linux_package;
pub mod linux_resolver;
pub mod linux_runtime;
pub mod virtio;
pub mod universal;
pub mod installer;
pub mod windows;

pub use android::AndroidLayer;
pub use bus::{DeviceId, DeviceKind, DriverBus};
pub use compat::{bind_compatible_driver, validate_contract, CompatibilityRegistry, DriverManifest, DriverSource};
pub use contract::{DeviceContract, DmaPolicy, InterruptMode, MmioRegion};
pub use core::{AdapterState, AndroidDriverAdapter, CoreError, DriverAdapter, DriverIdentity, DriverSlot, HardwareAbstraction, HardwareInfo, LinuxDriverAdapter, WindowsDriverAdapter};
pub use learning::{DriverExperience, ExperienceDb, ProbeOutcome};
pub use linux::LinuxLayer;
pub use linux_install::{plan, InstallError as LinuxInstallError, InstallPlan as LinuxInstallPlan};
pub use linux_package::{prepare_probe, validate_package, LinuxPackageError, LinuxPackageHeader, LDRIVER_MAGIC, MAX_PAYLOAD};
pub use linux_resolver::{resolve, LinuxCandidate, ResolveError};
pub use linux_runtime::{LinuxDriverDescriptor, LinuxRuntime, LinuxRuntimeError};
pub use virtio::{VirtioDevice, VirtioFeatures};
pub use universal::{validate_request, DriverAbi, DriverAction, DriverError, DriverOs, DriverRequest, DriverResult};
pub use installer::{plan_install, InstallError, InstallPlan, InstallerPackage, PackageFormat};
pub use windows::WindowsLayer;
