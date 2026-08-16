#![no_std]

use super::core::{CoreError, DriverAdapter, DriverSlot, HardwareInfo, LinuxDriverAdapter};
use super::universal::{DriverAbi, DriverOs};

pub struct LinuxLayer<A> { pub slot: DriverSlot<A> }
impl<A: LinuxDriverAdapter> LinuxLayer<A> {
    pub const fn new(adapter:A)->Self{Self{slot:DriverSlot::new(adapter)}}
    pub fn validate(&self,hw:&HardwareInfo)->Result<(),CoreError>{
        let id=self.slot.adapter.identity();
        if id.os!=DriverOs::Linux||(id.abi!=DriverAbi::LinuxKmod&&id.abi!=DriverAbi::LinuxUserMode){return Err(CoreError::UnsupportedAbi)}
        if !id.matches(hw){return Err(CoreError::InvalidDevice)}
        let api=self.slot.adapter.linux_api_version();
        if api==0||api>u32::from(u16::MAX)||id.api_version!=api as u16||self.slot.adapter.linux_module_name().is_empty(){return Err(CoreError::InvalidRequest)}
        if !hw.valid(){return Err(CoreError::InvalidDevice)}
        if hw.irq==u32::MAX{return Err(CoreError::InvalidDevice)}
        Ok(())
    }
}
