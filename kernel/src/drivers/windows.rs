#![no_std]

use super::core::{CoreError, DriverAdapter, DriverSlot, HardwareInfo, WindowsDriverAdapter};
use super::universal::{DriverAbi, DriverOs};

pub struct WindowsLayer<A>{pub slot:DriverSlot<A>}
impl<A:WindowsDriverAdapter> WindowsLayer<A>{
 pub const fn new(adapter:A)->Self{Self{slot:DriverSlot::new(adapter)}}
 pub fn validate(&self,hw:&HardwareInfo)->Result<(),CoreError>{
  let id=self.slot.adapter.identity();
  if id.os!=DriverOs::Windows||id.abi!=DriverAbi::WindowsCompat{return Err(CoreError::UnsupportedAbi)}
  let api=self.slot.adapter.windows_api_version();
  if id.api_version==0||api==0||api>u32::from(u16::MAX)||id.api_version!=api as u16{return Err(CoreError::InvalidRequest)}
  if self.slot.adapter.windows_driver_name().is_empty(){return Err(CoreError::InvalidRequest)}
  if !id.matches(hw)||!hw.valid(){return Err(CoreError::InvalidDevice)}
  Ok(())
 }
}

#[cfg(test)]
mod tests{
 use super::*; use super::super::bus::DeviceId;
 struct Mock{api:u32,name:&'static str,os:DriverOs,abi:DriverAbi,vendor:u16,device:u16,signed:bool}
 impl DriverAdapter for Mock{fn identity(&self)->super::super::core::DriverIdentity{super::super::core::DriverIdentity{os:self.os,abi:self.abi,api_version:self.api as u16,vendor:self.vendor,device:self.device,signed:self.signed}}fn probe(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}fn start(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}fn stop(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}fn remove(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}}
 impl WindowsDriverAdapter for Mock{fn windows_api_version(&self)->u32{self.api}fn windows_driver_name(&self)->&'static str{self.name}}
 fn hw()->HardwareInfo{HardwareInfo{id:DeviceId{vendor:1,device:2,class:0,revision:1},mmio_base:0x1000,mmio_length:0x100,irq:5,dma_bits:64}}
 fn good()->WindowsLayer<Mock>{WindowsLayer::new(Mock{api:7,name:"awe-wdm",os:DriverOs::Windows,abi:DriverAbi::WindowsCompat,vendor:1,device:2,signed:true})}
 #[test]fn valid_adapter_passes(){assert!(good().validate(&hw()).is_ok())}
 #[test]fn mismatched_identity_is_rejected(){let mut x=good();x.slot.adapter.vendor=9;assert_eq!(x.validate(&hw()),Err(CoreError::InvalidDevice))}
 #[test]fn api_mismatch_is_rejected(){let mut x=good();x.slot.adapter.api=8;assert_eq!(x.validate(&hw()),Err(CoreError::InvalidRequest))}
 #[test]fn unsigned_driver_is_rejected(){let mut x=good();x.slot.adapter.signed=false;assert_eq!(x.validate(&hw()),Err(CoreError::InvalidDevice))}
}