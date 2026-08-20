#![no_std]

use super::bus::DeviceId;
use super::contract::HardwareResource;
use super::linux_dependency::{Dependency, DependencyError, validate};
use super::linux_install::{InstallError, InstallPlan, plan};
use super::linux_package::LinuxPackageHeader;
use super::linux_resolver::LinuxCandidate;
use super::linux_runtime::LinuxRuntime;
use super::linux_transaction::{DriverTransaction, TransactionError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GuardError {
    Dependency(DependencyError),
    Install(InstallError),
    Transaction(TransactionError),
}

/// Dependency-aware transaction gate. A transaction cannot be prepared
/// until every declared direct dependency is present and valid.
pub fn prepare_guarded(
    device: DeviceId,
    candidates: &[LinuxCandidate],
    dependencies: &[Dependency],
    runtime: &LinuxRuntime,
    header: LinuxPackageHeader,
    resources: HardwareResource,
) -> Result<DriverTransaction, GuardError> {
    validate(candidates, dependencies).map_err(GuardError::Dependency)?;
    DriverTransaction::prepare(device, candidates, runtime, header, resources)
        .map_err(GuardError::Transaction)
}

pub fn install_plan_guarded(
    device: DeviceId,
    candidates: &[LinuxCandidate],
    dependencies: &[Dependency],
    runtime: &LinuxRuntime,
    header: LinuxPackageHeader,
    resources: HardwareResource,
) -> Result<InstallPlan, GuardError> {
    validate(candidates, dependencies).map_err(GuardError::Dependency)?;
    plan(device, candidates, runtime, header, resources).map_err(GuardError::Install)
}

#[cfg(test)]
mod tests {
    use super::super::linux_package::{LDRIVER_MAGIC, LinuxDriverDescriptor};
    use super::*;
    const DEV: DeviceId = DeviceId {
        vendor: 0x8086,
        device: 0x100e,
        class: 0x0200,
        revision: 1,
    };
    const RES: HardwareResource = HardwareResource {
        mmio_base: 0x1000,
        mmio_length: 0x1000,
        dma_mask: u64::MAX,
        irq: 11,
    };
    const DESC: LinuxDriverDescriptor = LinuxDriverDescriptor {
        vendor: 0x8086,
        device: 0x100e,
        class: 0x0200,
        api_version: 6,
        module_hash: 10,
        signed: true,
    };
    const HDR: LinuxPackageHeader = LinuxPackageHeader {
        magic: LDRIVER_MAGIC,
        format_version: 1,
        descriptor_size: core::mem::size_of::<LinuxDriverDescriptor>() as u16,
        payload_size: 4096,
        checksum: 1,
    };

    #[test]
    fn dependency_failure_blocks_prepare() {
        let candidate = LinuxCandidate {
            descriptor: DESC,
            priority: 10,
        };
        let result = prepare_guarded(
            DEV,
            &[candidate],
            &[Dependency {
                driver_hash: 10,
                required_hash: 99,
            }],
            &LinuxRuntime::new(6),
            HDR,
            RES,
        );
        assert_eq!(
            result,
            Err(GuardError::Dependency(DependencyError::Missing))
        );
    }

    #[test]
    fn valid_dependencies_allow_prepare() {
        let a = LinuxCandidate {
            descriptor: DESC,
            priority: 10,
        };
        let b = LinuxCandidate {
            descriptor: LinuxDriverDescriptor {
                module_hash: 20,
                ..DESC
            },
            priority: 5,
        };
        let result = prepare_guarded(
            DEV,
            &[a, b],
            &[Dependency {
                driver_hash: 10,
                required_hash: 20,
            }],
            &LinuxRuntime::new(6),
            HDR,
            RES,
        );
        assert!(result.is_ok());
    }
}
