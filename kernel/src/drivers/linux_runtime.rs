#![no_std]
use super::bus::DeviceId;
use super::contract::{DriverIdentity, HardwareResource, validate_hardware, validate_identity};
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
    pub const fn new(max_api_version: u32) -> Self {
        Self { max_api_version }
    }
    pub fn validate(
        &self,
        descriptor: LinuxDriverDescriptor,
        device: DeviceId,
        resources: HardwareResource,
    ) -> Result<(), LinuxRuntimeError> {
        if descriptor.vendor == 0xffff || descriptor.device == 0xffff || descriptor.module_hash == 0
        {
            return Err(LinuxRuntimeError::InvalidDescriptor);
        }
        if descriptor.api_version == 0 || descriptor.api_version > self.max_api_version {
            return Err(LinuxRuntimeError::UnsupportedApi);
        }
        if !descriptor.signed {
            return Err(LinuxRuntimeError::Unsigned);
        }
        if descriptor.vendor != device.vendor || descriptor.device != device.device {
            return Err(LinuxRuntimeError::DeviceMismatch);
        }
        let identity = DriverIdentity {
            os: DriverOs::Linux,
            abi: DriverAbi::LinuxKmod,
            api_version: descriptor.api_version,
            signed: descriptor.signed,
        };
        validate_identity(identity).map_err(|_| LinuxRuntimeError::InvalidDescriptor)?;
        validate_hardware(device, resources).map_err(|_| LinuxRuntimeError::InvalidHardware)?;
        Ok(())
    }
    pub fn probe_request(
        &self,
        descriptor: LinuxDriverDescriptor,
        device: DeviceId,
        resources: HardwareResource,
    ) -> Result<DriverRequest, LinuxRuntimeError> {
        self.validate(descriptor, device, resources)?;
        Ok(DriverRequest {
            device,
            os: DriverOs::Linux,
            abi: DriverAbi::LinuxKmod,
            action: DriverAction::Probe,
            version: descriptor.api_version,
            signed: true,
        })
    }
}
