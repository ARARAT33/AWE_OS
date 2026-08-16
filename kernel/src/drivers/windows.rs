#![no_std]

use super::core::{CoreError, DriverAdapter, DriverSlot, HardwareInfo, WindowsDriverAdapter};
use super::universal::{DriverAbi, DriverOs};

pub struct WindowsLayer<A> { pub slot: DriverSlot<A> }

impl<A: WindowsDriverAdapter> WindowsLayer<A> {
    pub const fn new(adapter: A) -> Self { Self { slot: DriverSlot::new(adapter) } }
    pub fn validate(&self, hw: &HardwareInfo) -> Result<(), CoreError> {
        let id = self.slot.adapter.identity();
        if id.os != DriverOs::Windows || id.abi != DriverAbi::WindowsCompat { return Err(CoreError::UnsupportedAbi); }
        if id.api_version == 0 || self.slot.adapter.windows_api_version() == 0 { return Err(CoreError::InvalidRequest); }
        if self.slot.adapter.windows_driver_name().is_empty() { return Err(CoreError::InvalidRequest); }
        if !id.matches(hw) || !hw.valid() { return Err(CoreError::InvalidDevice); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::bus::DeviceId;
    struct D { api:u32, name:&'static str }
    impl DriverAdapter for D { fn identity(&self)->super::super::core::DriverIdentity { super::super::core::DriverIdentity{os:DriverOs::Windows,abi:DriverAbi::WindowsCompat,api_version:self.api as u16,vendor:1,device:2,signed:true} } fn probe(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())} fn start(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())} fn stop(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())} fn remove(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())} }
    impl WindowsDriverAdapter for D { fn windows_api_version(&self)->u32{self.api} fn windows_driver_name(&self)->&'static str{self.name} }
    fn hw()->HardwareInfo { HardwareInfo{id:DeviceId{vendor:1,device:2,class:0,revision:1},mmio_base:0x1000,mmio_length:0x100,irq:1,dma_bits:64} }
    #[test] fn valid_adapter_passes(){assert!(WindowsLayer::new(D{api:1,name:"awe-wdm"}).validate(&hw()).is_ok());}
    #[test] fn missing_name_rejected(){assert_eq!(WindowsLayer::new(D{api:1,name:""}).validate(&hw()),Err(CoreError::InvalidRequest));}
    #[test] fn invalid_api_rejected(){assert_eq!(WindowsLayer::new(D{api:0,name:"awe-wdm"}).validate(&hw()),Err(CoreError::InvalidRequest));}
}
