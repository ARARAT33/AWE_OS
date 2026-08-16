#![no_std]

use super::bus::DeviceId;
use super::contract::{validate_hardware, validate_identity, DriverIdentity, HardwareResource};
use super::linux_runtime::{LinuxDriverDescriptor, LinuxRuntime, LinuxRuntimeError};
use super::universal::{DriverAbi, DriverAction, DriverOs, DriverRequest};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LinuxPackageError { InvalidHeader, InvalidLength, InvalidChecksum, Runtime(LinuxRuntimeError) }

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LinuxPackageHeader {
    pub magic: u32,
    pub format_version: u16,
    pub descriptor_size: u16,
    pub payload_size: u32,
    pub checksum: u64,
}

pub const LDRIVER_MAGIC: u32 = 0x4157_4452;
pub const MAX_PAYLOAD: u32 = 16 * 1024 * 1024;

pub fn validate_package(header: LinuxPackageHeader, descriptor: LinuxDriverDescriptor) -> Result<(), LinuxPackageError> {
    if header.magic != LDRIVER_MAGIC || header.format_version == 0 || header.descriptor_size as usize != core::mem::size_of::<LinuxDriverDescriptor>() {
        return Err(LinuxPackageError::InvalidHeader);
    }
    if header.payload_size == 0 || header.payload_size > MAX_PAYLOAD || header.checksum == 0 || descriptor.module_hash == 0 {
        return Err(LinuxPackageError::InvalidLength);
    }
    Ok(())
}

pub fn prepare_probe(runtime: &LinuxRuntime, header: LinuxPackageHeader, descriptor: LinuxDriverDescriptor, device: DeviceId, resources: HardwareResource) -> Result<DriverRequest, LinuxPackageError> {
    validate_package(header, descriptor)?;
    runtime.probe_request(descriptor, device, resources).map_err(LinuxPackageError::Runtime)
}

#[cfg(test)]
mod tests {
    use super::*;
    const DEV: DeviceId = DeviceId { vendor: 0x8086, device: 0x100e, class: 0x0200, revision: 1 };
    const RES: HardwareResource = HardwareResource { mmio_base: 0x1000, mmio_length: 0x1000, dma_mask: u64::MAX, irq: 11 };
    const DESC: LinuxDriverDescriptor = LinuxDriverDescriptor { vendor: 0x8086, device: 0x100e, class: 0x0200, api_version: 6, module_hash: 1, signed: true };
    const HDR: LinuxPackageHeader = LinuxPackageHeader { magic: LDRIVER_MAGIC, format_version: 1, descriptor_size: core::mem::size_of::<LinuxDriverDescriptor>() as u16, payload_size: 4096, checksum: 1 };

    #[test]
    fn valid_package_reaches_probe() { assert!(prepare_probe(&LinuxRuntime::new(6), HDR, DESC, DEV, RES).is_ok()); }
    #[test]
    fn malformed_magic_is_rejected() { let bad=LinuxPackageHeader{magic:0, ..HDR}; assert_eq!(validate_package(bad,DESC),Err(LinuxPackageError::InvalidHeader)); }
}
