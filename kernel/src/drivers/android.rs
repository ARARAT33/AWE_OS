#![no_std]

use super::core::{AndroidDriverAdapter, CoreError, DriverAdapter, DriverSlot, HardwareInfo};
use super::universal::{DriverAbi, DriverOs};

pub struct AndroidLayer<A>{pub slot:DriverSlot<A>}
impl<A:AndroidDriverAdapter> AndroidLayer<A>{
 pub const fn new(adapter:A)->Self{Self{slot:DriverSlot::new(adapter)}}
 pub fn validate(&self,hw:&HardwareInfo)->Result<(),CoreError>{
  let id=self.slot.adapter.identity();
  if id.os!=DriverOs::Android||id.abi!=DriverAbi::AndroidHal{return Err(CoreError::UnsupportedAbi)}
  let api=self.slot.adapter.android_hal_version();
  if id.api_version==0||api==0||api>u32::from(u16::MAX)||id.api_version!=api as u16{return Err(CoreError::InvalidRequest)}
  if self.slot.adapter.android_interface_name().is_empty(){return Err(CoreError::InvalidRequest)}
  if !id.matches(hw)||!hw.valid(){return Err(CoreError::InvalidDevice)}
  Ok(())
 }
}
