#![no_std]
use super::core::{AdapterState,CoreError,DriverAdapter,DriverSlot,HardwareInfo,WindowsDriverAdapter};
use super::universal::{DriverAbi,DriverOs};

pub struct WindowsLayer<A>{pub slot:DriverSlot<A>}
impl<A:WindowsDriverAdapter>WindowsLayer<A>{
 pub const fn new(adapter:A)->Self{Self{slot:DriverSlot::new(adapter)}}
 pub fn validate(&self,hw:&HardwareInfo)->Result<(),CoreError>{
  let id=self.slot.adapter.identity();
  if id.os!=DriverOs::Windows||id.abi!=DriverAbi::WindowsCompat{return Err(CoreError::UnsupportedAbi)}
  let api=self.slot.adapter.windows_api_version();
  if id.api_version==0||api==0||api>u32::from(u16::MAX)||id.api_version!=api as u16{return Err(CoreError::InvalidRequest)}
  let name=self.slot.adapter.windows_driver_name();
  if name.len()<3||!name.as_bytes().iter().any(|b|*b==b'-'){return Err(CoreError::InvalidRequest)}
  if !id.matches(hw)||!hw.valid()||hw.irq==u32::MAX{return Err(CoreError::InvalidDevice)}
  Ok(())
 }
 pub fn can_probe(&self,hw:&HardwareInfo)->bool{self.validate(hw).is_ok()&&self.slot.state==AdapterState::New}
 pub fn probe(&mut self,hw:&HardwareInfo)->Result<(),CoreError>{self.validate(hw)?;if self.slot.state!=AdapterState::New{return Err(CoreError::NotBound)}self.slot.adapter.probe(hw).map_err(|_|CoreError::ProbeFailed)?;self.slot.state=AdapterState::Probed;Ok(())}
 pub fn start(&mut self,hw:&HardwareInfo)->Result<(),CoreError>{self.validate(hw)?;if self.slot.state!=AdapterState::Probed&&self.slot.state!=AdapterState::Stopped{return Err(CoreError::NotBound)}self.slot.adapter.start(hw).map_err(|_|CoreError::StartFailed)?;self.slot.state=AdapterState::Running;Ok(())}
 pub fn stop(&mut self,hw:&HardwareInfo)->Result<(),CoreError>{if self.slot.state!=AdapterState::Running{return Err(CoreError::NotBound)}self.slot.adapter.stop(hw).map_err(|_|CoreError::PolicyDenied)?;self.slot.state=AdapterState::Stopped;Ok(())}
 pub fn remove(&mut self,hw:&HardwareInfo)->Result<(),CoreError>{if self.slot.state==AdapterState::Running{return Err(CoreError::PolicyDenied)}if self.slot.state==AdapterState::Removed{return Err(CoreError::NotBound)}self.slot.adapter.remove(hw).map_err(|_|CoreError::PolicyDenied)?;self.slot.state=AdapterState::Removed;Ok(())}
}

#[cfg(test)]mod tests{use super::*;use super::super::bus::DeviceId;
struct Mock{api:u32,name:&'static str,os:DriverOs,abi:DriverAbi,vendor:u16,device:u16,signed:bool,fail_probe:bool,fail_start:bool}
impl DriverAdapter for Mock{fn identity(&self)->super::super::core::DriverIdentity{super::super::core::DriverIdentity{os:self.os,abi:self.abi,api_version:self.api as u16,vendor:self.vendor,device:self.device,signed:self.signed}}fn probe(&mut self,_:&HardwareInfo)->Result<(),CoreError>{if self.fail_probe{Err(CoreError::ProbeFailed)}else{Ok(())}}fn start(&mut self,_:&HardwareInfo)->Result<(),CoreError>{if self.fail_start{Err(CoreError::StartFailed)}else{Ok(())}}fn stop(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}fn remove(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}}
impl WindowsDriverAdapter for Mock{fn windows_api_version(&self)->u32{self.api}fn windows_driver_name(&self)->&'static str{self.name}}
fn hw()->HardwareInfo{HardwareInfo{id:DeviceId{vendor:1,device:2,class:0,revision:1},mmio_base:0x1000,mmio_length:0x100,irq:5,dma_bits:64}}
fn good()->WindowsLayer<Mock>{WindowsLayer::new(Mock{api:7,name:"awe-wdm",os:DriverOs::Windows,abi:DriverAbi::WindowsCompat,vendor:1,device:2,signed:true,fail_probe:false,fail_start:false})}
#[test]fn full_lifecycle_executes(){let mut x=good();x.probe(&hw()).unwrap();x.start(&hw()).unwrap();x.stop(&hw()).unwrap();x.remove(&hw()).unwrap();assert_eq!(x.slot.state,AdapterState::Removed)}
#[test]fn failed_probe_does_not_advance_state(){let mut x=WindowsLayer::new(Mock{api:7,name:"awe-wdm",os:DriverOs::Windows,abi:DriverAbi::WindowsCompat,vendor:1,device:2,signed:true,fail_probe:true,fail_start:false});assert_eq!(x.probe(&hw()),Err(CoreError::ProbeFailed));assert_eq!(x.slot.state,AdapterState::New)}
}