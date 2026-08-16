#![no_std]

use super::core::{CoreError, DriverAdapter, DriverSlot, HardwareInfo, AndroidDriverAdapter};
use super::universal::{DriverAbi, DriverOs};

/// Android HAL/vendor compatibility boundary. Android interfaces are translated
/// into the same AWE driver lifecycle and hardware contract as every other OS.
pub struct AndroidLayer<A> { pub slot: DriverSlot<A> }

impl<A: AndroidDriverAdapter> AndroidLayer<A> {
    pub const fn new(adapter: A) -> Self { Self { slot: DriverSlot::new(adapter) } }
    pub fn validate(&self, hw: &HardwareInfo) -> Result<(), CoreError> {
        let id = self.slot.adapter.identity();
        if id.os != DriverOs::Android || id.abi != DriverAbi::AndroidHal { return Err(CoreError::UnsupportedAbi); }
        if !id.matches(hw) { return Err(CoreError::InvalidDevice); }
        if self.slot.adapter.android_hal_version() == 0 { return Err(CoreError::InvalidRequest); }
        Ok(())
    }
}
