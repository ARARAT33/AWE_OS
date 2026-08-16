#![no_std]

use super::core::{CoreError, DriverAdapter, DriverIdentity, DriverSlot, HardwareAbstraction, HardwareInfo, LinuxDriverAdapter};
use super::universal::{DriverAbi, DriverOs};

pub struct LinuxLayer<A> { pub slot: DriverSlot<A> }
impl<A: LinuxDriverAdapter> LinuxLayer<A> {
    pub const fn new(adapter: A) -> Self { Self { slot: DriverSlot::new(adapter) } }
    pub fn validate(&self, hw: &HardwareInfo) -> Result<(), CoreError> {
        let id = self.slot.adapter.identity();
        if id.os != DriverOs::Linux || (id.abi != DriverAbi::LinuxKmod && id.abi != DriverAbi::LinuxUserMode) { return Err(CoreError::UnsupportedAbi); }
        if !id.matches(hw) { return Err(CoreError::InvalidDevice); }
        if self.slot.adapter.linux_api_version() == 0 || self.slot.adapter.linux_module_name().is_empty() { return Err(CoreError::InvalidRequest); }
        if !hw.valid() || hw.mmio_base.checked_add(hw.mmio_length).is_none() { return Err(CoreError::InvalidDevice); }
        Ok(())
    }
}
