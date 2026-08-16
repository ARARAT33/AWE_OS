#![no_std]
use super::bus::DeviceId;
use super::linux_runtime::LinuxDriverDescriptor;
#[derive(Clone,Copy,PartialEq,Eq,Debug)]pub struct LinuxCandidate{pub descriptor:LinuxDriverDescriptor,pub priority:u8}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]pub enum ResolveError{NoMatch,Ambiguous}
pub fn resolve(device:DeviceId,candidates:&[LinuxCandidate])->Result<LinuxCandidate,ResolveError>{let mut best=None;let mut ties=0u8;for candidate in candidates{let d=candidate.descriptor;if d.vendor!=device.vendor||d.device!=device.device||d.class as u32!=device.class||!d.signed||d.module_hash==0{continue}match best{None=>{best=Some(*candidate);ties=1}Some(current)if candidate.priority>current.priority=>{best=Some(*candidate);ties=1}Some(current)if candidate.priority==current.priority=>{if d.api_version>current.descriptor.api_version{best=Some(*candidate);ties=1}else if d.api_version==current.descriptor.api_version{ties=ties.saturating_add(1)}}_=>{}}}match best{None=>Err(ResolveError::NoMatch),Some(_)if ties>1=>Err(ResolveError::Ambiguous),Some(candidate)=>Ok(candidate)}}
