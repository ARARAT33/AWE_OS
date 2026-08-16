#![no_std]

use super::core::{CoreError, DriverAdapter, DriverIdentity, DriverSlot, HardwareAbstraction, HardwareInfo, LinuxDriverAdapter};
use super::universal::{DriverAbi, DriverOs};

/// Linux compatibility layer. A native Linux driver is represented by an
/// adapter; its kernel-facing operations must be translated to AWE HAL calls.
pub struct LinuxLayer<A> { pub slot: DriverSlot<A> }

impl<A: LinuxDriverAdapter> LinuxLayer<A> {
    pub const fn new(adapter: A) -> Self { Self { slot: DriverSlot::new(adapter) } }
    pub fn validate(&self, hw: &HardwareInfo) -> Result<(), CoreError> {
        let id = self.slot.adapter.identity();
        if id.os != DriverOs::Linux || id.abi != DriverAbi::LinuxKmod && id.abi != DriverAbi::LinuxUserMode { return Err(CoreError::UnsupportedAbi); }
        if !id.matches(hw) { return Err(CoreError::InvalidDevice); }
        if self.slot.adapter.linux_api_version() == 0 { return Err(CoreError::InvalidRequest); }
        Ok(())
    }
}
