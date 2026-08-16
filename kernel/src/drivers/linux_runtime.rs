#![no_std]

use super::bus::DeviceId;
use super::contract::{validate_hardware, validate_identity, DriverIdentity, HardwareResource};
use super::universal::{DriverAbi, DriverAction, DriverOs, DriverRequest};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LinuxDriverDescriptor {
    pub vendor: u16,
    pub device: u16,
    pub class: u16,
    pub api_version: u32,
    pub module_hash: u64,
    pub signed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LinuxRuntimeError {
    InvalidDescriptor,
    UnsupportedApi,
    Unsigned,
    DeviceMismatch,
    InvalidHardware,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LinuxRuntime {
    pub max_api_version: u32,
}

impl LinuxRuntime {
    pub const fn new(max_api_version: u32) -> Self { Self { max_api_version } }

    pub fn validate(&self, descriptor: LinuxDriverDescriptor, device: DeviceId, resources: HardwareResource) -> Result<(), LinuxRuntimeError> {
        if descriptor.vendor == 0xffff || descriptor.device == 0xffff || descriptor.module_hash == 0 {
            return Err(LinuxRuntimeError::InvalidDescriptor);
        }
        if descriptor.api_version == 0 || descriptor.api_version > self.max_api_version {
            return Err(LinuxRuntimeError::UnsupportedApi);
        }
        if !descriptor.signed { return Err(LinuxRuntimeError::Unsigned); }
        if descriptor.vendor != device.vendor || descriptor.device != device.device {
            return Err(LinuxRuntimeError::DeviceMismatch);
        }
        let identity = DriverIdentity { os: DriverOs::Linux, abi: DriverAbi::LinuxKmod, api_version: descriptor.api_version, signed: descriptor.signed };
        validate_identity(identity).map_err(|_| LinuxRuntimeError::InvalidDescriptor)?;
        validate_hardware(device, resources).map_err(|_| LinuxRuntimeError::InvalidHardware)?;
        Ok(())
    }

    pub fn probe_request(&self, descriptor: LinuxDriverDescriptor, device: DeviceId, resources: HardwareResource) -> Result<DriverRequest, LinuxRuntimeError> {
        self.validate(descriptor, device, resources)?;
        Ok(DriverRequest { device, os: DriverOs::Linux, abi: DriverAbi::LinuxKmod, action: DriverAction::Probe, version: descriptor.api_version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DEV: DeviceId = DeviceId { vendor: 0x8086, device: 0x100e, class: 0x0200, revision: 1 };
    const RES: HardwareResource = HardwareResource { mmio_base: 0x1000, mmio_length: 0x1000, dma_mask: u64::MAX, irq: 11 };

    #[test]
    fn matching_signed_module_can_probe() {
        let runtime = LinuxRuntime::new(6);
        let descriptor = LinuxDriverDescriptor { vendor: 0x8086, device: 0x100e, class: 0x0200, api_version: 6, module_hash: 1, signed: true };
        assert!(runtime.probe_request(descriptor, DEV, RES).is_ok());
    }

    #[test]
    fn wrong_device_is_rejected() {
        let runtime = LinuxRuntime::new(6);
        let descriptor = LinuxDriverDescriptor { vendor: 0x10ec, device: 0x8168, class: 0x0200, api_version: 6, module_hash: 1, signed: true };
        assert_eq!(runtime.validate(descriptor, DEV, RES), Err(LinuxRuntimeError::DeviceMismatch));
    }
}
