#![no_std]
use super::core::{AdapterState,AndroidDriverAdapter,CoreError,DriverAdapter,DriverSlot,HardwareAbstraction,HardwareInfo};
use super::universal::{DriverAbi,DriverOs};
pub struct AndroidLayer<A>{pub slot:DriverSlot<A>}
impl<A:AndroidDriverAdapter>AndroidLayer<A>{
 pub const fn new(adapter:A)->Self{Self{slot:DriverSlot::new(adapter)}}
 pub fn validate(&self,hw:&HardwareInfo)->Result<(),CoreError>{let id=self.slot.adapter.identity();if id.os!=DriverOs::Android||id.abi!=DriverAbi::AndroidHal{return Err(CoreError::UnsupportedAbi)}let api=self.slot.adapter.android_hal_version();if id.api_version==0||api==0||api>u32::from(u16::MAX)||id.api_version!=api as u16{return Err(CoreError::InvalidRequest)}let name=self.slot.adapter.android_interface_name();if name.len()<3||!name.as_bytes().iter().any(|b|*b==b'/'){return Err(CoreError::InvalidRequest)}if !id.matches(hw)||!hw.valid()||hw.irq==u32::MAX{return Err(CoreError::InvalidDevice)}Ok(())}
 pub fn can_probe(&self,hw:&HardwareInfo)->bool{self.validate(hw).is_ok()&&self.slot.state==AdapterState::New}
 pub fn probe(&mut self,hw:&HardwareInfo)->Result<(),CoreError>{self.validate(hw)?;if self.slot.state!=AdapterState::New{return Err(CoreError::NotBound)}self.slot.adapter.probe(hw).map_err(|_|CoreError::ProbeFailed)?;self.slot.state=AdapterState::Probed;Ok(())}
 pub fn start(&mut self,hw:&HardwareInfo)->Result<(),CoreError>{self.validate(hw)?;if self.slot.state!=AdapterState::Probed&&self.slot.state!=AdapterState::Stopped{return Err(CoreError::NotBound)}self.slot.adapter.start(hw).map_err(|_|CoreError::StartFailed)?;self.slot.state=AdapterState::Running;Ok(())}
 pub fn stop(&mut self,hw:&HardwareInfo)->Result<(),CoreError>{if self.slot.state!=AdapterState::Running{return Err(CoreError::NotBound)}self.slot.adapter.stop(hw).map_err(|_|CoreError::PolicyDenied)?;self.slot.state=AdapterState::Stopped;Ok(())}
 pub fn remove(&mut self,hw:&HardwareInfo)->Result<(),CoreError>{if self.slot.state==AdapterState::Running{return Err(CoreError::PolicyDenied)}if self.slot.state==AdapterState::Removed{return Err(CoreError::NotBound)}self.slot.adapter.remove(hw).map_err(|_|CoreError::PolicyDenied)?;self.slot.state=AdapterState::Removed;Ok(())}
 pub fn mmio_read32<H:HardwareAbstraction>(&self,hw:&HardwareInfo,hal:&H,offset:u64)->Result<u32,CoreError>{if self.slot.state!=AdapterState::Running{return Err(CoreError::NotBound)}self.slot.mmio_read32(hw,hal,offset)}
 pub fn mmio_write32<H:HardwareAbstraction>(&self,hw:&HardwareInfo,hal:&mut H,offset:u64,value:u32)->Result<(),CoreError>{if self.slot.state!=AdapterState::Running{return Err(CoreError::NotBound)}self.slot.mmio_write32(hw,hal,offset,value)}
 pub fn irq_ack<H:HardwareAbstraction>(&self,hw:&HardwareInfo,hal:&mut H)->Result<(),CoreError>{if self.slot.state!=AdapterState::Running{return Err(CoreError::NotBound)}self.slot.irq_ack(hw,hal)}
 pub fn dma_submit<H:HardwareAbstraction>(&self,hw:&HardwareInfo,hal:&mut H,bytes:u64,address_bits:u8)->Result<(),CoreError>{if self.slot.state!=AdapterState::Running{return Err(CoreError::NotBound)}self.slot.dma_submit(hw,hal,bytes,address_bits)}
}
