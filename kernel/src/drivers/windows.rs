#![no_std]

use super::core::{CoreError, DriverAdapter, DriverSlot, HardwareInfo, WindowsDriverAdapter};
use super::universal::{DriverAbi, DriverOs};

/// Windows compatibility boundary for WDM/KMDF-style adapters. It never
/// executes a foreign kernel binary directly inside CellKernel.
pub struct WindowsLayer<A> { pub slot: DriverSlot<A> }

impl<A: WindowsDriverAdapter> WindowsLayer<A> {
    pub const fn new(adapter: A) -> Self { Self { slot: DriverSlot::new(adapter) } }
    pub fn validate(&self, hw: &HardwareInfo) -> Result<(), CoreError> {
        let id = self.slot.adapter.identity();
        if id.os != DriverOs::Windows || id.abi != DriverAbi::WindowsCompat { return Err(CoreError::UnsupportedAbi); }
        if !id.matches(hw) { return Err(CoreError::InvalidDevice); }
        if self.slot.adapter.windows_api_version() == 0 { return Err(CoreError::InvalidRequest); }
        Ok(())
    }
}
