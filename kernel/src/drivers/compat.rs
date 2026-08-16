#![no_std]

use super::{DeviceContract, DeviceId, DeviceKind, DriverBus};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DriverSource { NativeAwe=0, LinuxPort=1, AndroidPort=2, WindowsPort=3, BsdPort=4, OtherPort=5 }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DriverManifest { pub vendor:u16, pub device:u16, pub class_code:u32, pub source:DriverSource, pub api_version:u16, pub verified:bool }
impl DriverManifest {
    pub const API_VERSION:u16=1;
    pub const fn matches(&self, contract:&DeviceContract<1>)->bool { self.vendor==contract.vendor && self.device==contract.device && self.class_code==contract.class_code && self.api_version==Self::API_VERSION }
    pub const fn can_load(&self)->bool { self.verified && self.api_version==Self::API_VERSION }
}

pub struct CompatibilityRegistry<const N:usize>{ entries:[Option<DriverManifest>;N] }
impl<const N:usize> CompatibilityRegistry<N>{
    pub const fn new()->Self{Self{entries:[None;N]}}
    pub fn register(&mut self, manifest:DriverManifest)->bool{if !manifest.can_load(){return false;}let mut i=0;while i<N{if self.entries[i].is_none(){self.entries[i]=Some(manifest);return true;}i+=1;}false}
    pub fn find(&self,vendor:u16,device:u16,class_code:u32)->Option<DriverManifest>{let mut i=0;while i<N{if let Some(m)=self.entries[i]{if m.vendor==vendor&&m.device==device&&m.class_code==class_code{return Some(m);}}i+=1;}None}
}

pub const fn validate_contract<const M:usize>(contract:&DeviceContract<M>)->bool{contract.valid()}

pub fn bind_compatible_driver<const N:usize,const M:usize>(bus:&mut DriverBus<N,M>,kind:DeviceKind,manifest:DriverManifest,contract:DeviceContract<M>)->bool{
    if !manifest.can_load()||manifest.vendor!=contract.vendor||manifest.device!=contract.device||manifest.class_code!=contract.class_code||!contract.valid(){return false;}
    bus.register(kind,contract).is_some()
}

#[cfg(test)]
mod tests{
 use super::*;use crate::drivers::{DmaPolicy,InterruptMode,MmioRegion};
 fn contract()->DeviceContract<1>{DeviceContract{vendor:0x1af4,device:1,class_code:0x0200,mmio:[Some(MmioRegion{base:0x1000,length:0x1000})],interrupt:InterruptMode::Msi,dma:DmaPolicy{max_bytes:4096,address_bits:48,coherent:true}}}
 #[test]fn only_verified_manifest_registers(){let mut r:CompatibilityRegistry<4>=CompatibilityRegistry::new();let good=DriverManifest{vendor:0x1af4,device:1,class_code:0x0200,source:DriverSource::NativeAwe,api_version:1,verified:true};assert!(!r.register(DriverManifest{verified:false,..good}));assert!(r.register(good));assert!(r.find(0x1af4,1,0x0200).is_some());}
 #[test]fn binding_rejects_unverified_driver(){let mut bus:DriverBus<4,1>=DriverBus::new();let m=DriverManifest{vendor:0x1af4,device:1,class_code:0x0200,source:DriverSource::LinuxPort,api_version:1,verified:false};assert!(!bind_compatible_driver(&mut bus,DeviceKind::Virtio,m,contract()));}
 #[test]fn binding_accepts_verified_matching_driver(){let mut bus:DriverBus<4,1>=DriverBus::new();let m=DriverManifest{vendor:0x1af4,device:1,class_code:0x0200,source:DriverSource::LinuxPort,api_version:1,verified:true};assert!(bind_compatible_driver(&mut bus,DeviceKind::Virtio,m,contract()));assert_eq!(bus.len(),1);}
}
