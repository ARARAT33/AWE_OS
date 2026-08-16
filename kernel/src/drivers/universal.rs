#![no_std]
use super::bus::DeviceId;
#[derive(Clone,Copy,PartialEq,Eq,Debug)]pub enum DriverOs{Linux,Android,Windows,Bsd,AweNative,Generic}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]pub enum DriverAbi{Native,LinuxKmod,LinuxUserMode,AndroidHal,WindowsCompat,Generic}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]pub enum DriverAction{Probe,Bind,Start,Stop,Remove}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]pub enum DriverError{UnsupportedAbi,InvalidDevice,NotSigned,VersionMismatch,PolicyDenied,InvalidAction}
#[repr(C)]#[derive(Clone,Copy)]pub struct DriverRequest{pub device:DeviceId,pub os:DriverOs,pub abi:DriverAbi,pub action:DriverAction,pub version:u32,pub signed:bool}
#[repr(C)]#[derive(Clone,Copy)]pub struct DriverResult{pub accepted:bool,pub score:u16,pub error:Option<DriverError>}
const MIN_VERSION:u32=1;const MAX_VERSION:u32=0xffff;
const fn compatible(os:DriverOs,abi:DriverAbi)->bool{match os{DriverOs::Linux=>matches!(abi,DriverAbi::LinuxKmod|DriverAbi::LinuxUserMode|DriverAbi::Generic),DriverOs::Android=>matches!(abi,DriverAbi::AndroidHal|DriverAbi::Generic),DriverOs::Windows=>matches!(abi,DriverAbi::WindowsCompat|DriverAbi::Generic),DriverOs::AweNative=>matches!(abi,DriverAbi::Native|DriverAbi::Generic),DriverOs::Bsd=>matches!(abi,DriverAbi::Generic),DriverOs::Generic=>true}}
pub const fn validate_request(r:&DriverRequest)->DriverResult{
 if r.version<MIN_VERSION||r.version>MAX_VERSION{return DriverResult{accepted:false,score:0,error:Some(DriverError::VersionMismatch)}}
 if !r.signed{return DriverResult{accepted:false,score:0,error:Some(DriverError::NotSigned)}}
 if r.device.0==0||r.device.0==u64::MAX{return DriverResult{accepted:false,score:0,error:Some(DriverError::InvalidDevice)}}
 if !compatible(r.os,r.abi){return DriverResult{accepted:false,score:0,error:Some(DriverError::UnsupportedAbi)}}
 if matches!(r.action,DriverAction::Start|DriverAction::Stop|DriverAction::Remove)&&matches!(r.abi,DriverAbi::Generic){return DriverResult{accepted:false,score:0,error:Some(DriverError::InvalidAction)}}
 let score=match r.action{DriverAction::Probe|DriverAction::Bind=>100,DriverAction::Start|DriverAction::Stop=>95,DriverAction::Remove=>90};DriverResult{accepted:true,score,error:None}
}
#[cfg(test)]mod tests{use super::*;fn dev()->DeviceId{DeviceId(0x8086_100e)}fn req()->DriverRequest{DriverRequest{device:dev(),os:DriverOs::Linux,abi:DriverAbi::LinuxUserMode,action:DriverAction::Probe,version:1,signed:true}}#[test]fn valid_request(){assert!(validate_request(&req()).accepted)}#[test]fn rejects_generic_lifecycle_without_adapter(){let mut r=req();r.abi=DriverAbi::Generic;r.action=DriverAction::Start;assert_eq!(validate_request(&r).error,Some(DriverError::InvalidAction))}#[test]fn rejects_unsigned(){let mut r=req();r.signed=false;assert_eq!(validate_request(&r).error,Some(DriverError::NotSigned))}#[test]fn boundary_versions(){let mut r=req();r.version=MIN_VERSION;assert!(validate_request(&r).accepted);r.version=MAX_VERSION;assert!(validate_request(&r).accepted)}}
