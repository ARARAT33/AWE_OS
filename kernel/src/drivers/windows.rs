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
  let name=self.slot.adapter.windows_driver_name();
  if name.is_empty(){return Err(CoreError::InvalidRequest)}
  if !id.matches(hw)||!hw.valid(){return Err(CoreError::InvalidDevice)}
  Ok(())
 }
}
