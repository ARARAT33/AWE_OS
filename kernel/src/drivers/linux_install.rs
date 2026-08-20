#![no_std]

use super::bus::DeviceId;
use super::contract::HardwareResource;
use super::linux_package::{LinuxPackageError, LinuxPackageHeader, prepare_probe};
use super::linux_resolver::{LinuxCandidate, ResolveError, resolve};
use super::linux_runtime::LinuxRuntime;
#[cfg(test)]
use super::linux_runtime::LinuxDriverDescriptor;
use super::universal::DriverRequest;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InstallError {
    Resolve(ResolveError),
    Package(LinuxPackageError),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InstallPlan {
    pub candidate: LinuxCandidate,
    pub probe: DriverRequest,
}

pub fn plan(
    device: DeviceId,
    candidates: &[LinuxCandidate],
    runtime: &LinuxRuntime,
    header: LinuxPackageHeader,
    resources: HardwareResource,
) -> Result<InstallPlan, InstallError> {
    let candidate = resolve(device, candidates).map_err(InstallError::Resolve)?;
    let probe = prepare_probe(runtime, header, candidate.descriptor, device, resources)
        .map_err(InstallError::Package)?;
    Ok(InstallPlan { candidate, probe })
}

#[cfg(test)]
mod tests {
    use super::super::linux_package::LDRIVER_MAGIC;
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
        module_hash: 1,
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
    fn plan_requires_exact_hardware_match() {
        let c = LinuxCandidate {
            descriptor: DESC,
            priority: 10,
        };
        assert!(plan(DEV, &[c], &LinuxRuntime::new(6), HDR, RES).is_ok());
    }
}
