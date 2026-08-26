#![no_std]

use super::{MountEntry, PackageFileRecord, SnapshotMetadata, StorageDaemon, StorageVolume, VolumeType, CACHE_BLOCKS, MAX_MOUNTS, MAX_PACKAGE_FILES, MAX_SNAPSHOTS, MAX_VOLUMES};

pub const STATE_MAGIC: [u8; 4] = *b"AWSP";
pub const STATE_VERSION: u8 = 1;
pub const MAX_STATE_SIZE: usize = 4096;
const HEADER_LEN: usize = 20;
const VOLUME_REC_LEN: usize = 24;
const MOUNT_REC_LEN: usize = 17;
const SNAPSHOT_REC_LEN: usize = 24;
const FILE_REC_LEN: usize = 25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistError { BufferTooSmall, Truncated, BadMagic, UnsupportedVersion, ChecksumMismatch, InvalidRecord, InvalidCount, Overflow }

fn checksum(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5u32;
    for &byte in bytes { hash ^= byte as u32; hash = hash.wrapping_mul(0x0100_0193); }
    hash
}
fn put_u16(out:&mut [u8],pos:&mut usize,value:u16)->Result<(),PersistError>{let end=pos.checked_add(2).ok_or(PersistError::Overflow)?;if end>out.len(){return Err(PersistError::BufferTooSmall)}out[*pos..end].copy_from_slice(&value.to_le_bytes());*pos=end;Ok(())}
fn put_u32(out:&mut [u8],pos:&mut usize,value:u32)->Result<(),PersistError>{let end=pos.checked_add(4).ok_or(PersistError::Overflow)?;if end>out.len(){return Err(PersistError::BufferTooSmall)}out[*pos..end].copy_from_slice(&value.to_le_bytes());*pos=end;Ok(())}
fn put_u64(out:&mut [u8],pos:&mut usize,value:u64)->Result<(),PersistError>{let end=pos.checked_add(8).ok_or(PersistError::Overflow)?;if end>out.len(){return Err(PersistError::BufferTooSmall)}out[*pos..end].copy_from_slice(&value.to_le_bytes());*pos=end;Ok(())}
fn put_u8(out:&mut [u8],pos:&mut usize,value:u8)->Result<(),PersistError>{if *pos>=out.len(){return Err(PersistError::BufferTooSmall)}out[*pos]=value;*pos+=1;Ok(())}
fn get_u16(input:&[u8],pos:&mut usize)->Result<u16,PersistError>{let end=pos.checked_add(2).ok_or(PersistError::Overflow)?;if end>input.len(){return Err(PersistError::Truncated)}let v=u16::from_le_bytes([input[*pos],input[*pos+1]]);*pos=end;Ok(v)}
fn get_u32(input:&[u8],pos:&mut usize)->Result<u32,PersistError>{let end=pos.checked_add(4).ok_or(PersistError::Overflow)?;if end>input.len(){return Err(PersistError::Truncated)}let v=u32::from_le_bytes([input[*pos],input[*pos+1],input[*pos+2],input[*pos+3]]);*pos=end;Ok(v)}
fn get_u64(input:&[u8],pos:&mut usize)->Result<u64,PersistError>{let end=pos.checked_add(8).ok_or(PersistError::Overflow)?;if end>input.len(){return Err(PersistError::Truncated)}let v=u64::from_le_bytes([input[*pos],input[*pos+1],input[*pos+2],input[*pos+3],input[*pos+4],input[*pos+5],input[*pos+6],input[*pos+7]]);*pos=end;Ok(v)}
fn get_u8(input:&[u8],pos:&mut usize)->Result<u8,PersistError>{if *pos>=input.len(){return Err(PersistError::Truncated)}let v=input[*pos];*pos+=1;Ok(v)}
fn volume_type_to_u8(v:VolumeType)->u8{match v{VolumeType::Ramdisk=>0,VolumeType::GptPartition=>1,VolumeType::VirtualDisk=>2,VolumeType::AweFsVolume=>3}}
fn volume_type_from_u8(v:u8)->Result<VolumeType,PersistError>{match v{0=>Ok(VolumeType::Ramdisk),1=>Ok(VolumeType::GptPartition),2=>Ok(VolumeType::VirtualDisk),3=>Ok(VolumeType::AweFsVolume),_=>Err(PersistError::InvalidRecord)}}

pub fn export_state(storage:&StorageDaemon,out:&mut [u8])->Result<usize,PersistError>{
    if out.len()<HEADER_LEN{return Err(PersistError::BufferTooSmall)}
    let vc=storage.volumes.iter().filter(|v|v.is_some()).count();let mc=storage.mounts.iter().filter(|v|v.is_some()).count();let sc=storage.snapshots.iter().filter(|v|v.is_some()).count();let fc=storage.files.iter().filter(|v|v.is_some()).count();
    if vc>MAX_VOLUMES||mc>MAX_MOUNTS||sc>MAX_SNAPSHOTS||fc>MAX_PACKAGE_FILES{return Err(PersistError::InvalidCount)}
    let payload_len=vc.checked_mul(VOLUME_REC_LEN).and_then(|v|v.checked_add(mc.checked_mul(MOUNT_REC_LEN)?)).and_then(|v|v.checked_add(sc.checked_mul(SNAPSHOT_REC_LEN)?)).and_then(|v|v.checked_add(fc.checked_mul(FILE_REC_LEN)?)).ok_or(PersistError::Overflow)?;
    let total_len=HEADER_LEN.checked_add(payload_len).ok_or(PersistError::Overflow)?;
    if total_len>out.len()||total_len>MAX_STATE_SIZE||total_len>u16::MAX as usize{return Err(PersistError::BufferTooSmall)}
    out[..total_len].fill(0);out[..4].copy_from_slice(&STATE_MAGIC);out[4]=STATE_VERSION;out[5]=0;out[6..8].copy_from_slice(&(total_len as u16).to_le_bytes());
    let mut pos=12usize;put_u16(out,&mut pos,vc as u16)?;put_u16(out,&mut pos,mc as u16)?;put_u16(out,&mut pos,sc as u16)?;put_u16(out,&mut pos,fc as u16)?;
    for r in storage.volumes.iter().flatten(){put_u32(out,&mut pos,r.volume_id)?;put_u8(out,&mut pos,volume_type_to_u8(r.volume_type))?;put_u8(out,&mut pos,u8::from(r.read_only))?;put_u16(out,&mut pos,0)?;put_u64(out,&mut pos,r.block_count)?;put_u64(out,&mut pos,r.start_lba)?}
    for r in storage.mounts.iter().flatten(){put_u32(out,&mut pos,r.mount_id)?;put_u32(out,&mut pos,r.volume_id)?;put_u64(out,&mut pos,r.path_hash)?;put_u8(out,&mut pos,u8::from(r.active))?}
    for r in storage.snapshots.iter().flatten(){put_u32(out,&mut pos,r.snapshot_id)?;put_u32(out,&mut pos,r.volume_id)?;put_u64(out,&mut pos,r.timestamp)?;put_u32(out,&mut pos,r.block_delta_count)?;put_u32(out,&mut pos,0)?}
    for r in storage.files.iter().flatten(){put_u32(out,&mut pos,r.file_id)?;put_u32(out,&mut pos,r.volume_id)?;put_u64(out,&mut pos,r.name_hash)?;put_u32(out,&mut pos,r.size_bytes)?;put_u8(out,&mut pos,u8::from(r.is_installed_package))?}
    out[8..12].copy_from_slice(&checksum(&out[HEADER_LEN..total_len]).to_le_bytes());Ok(total_len)
}

pub fn import_state(storage:&mut StorageDaemon,input:&[u8])->Result<(),PersistError>{
    if input.len()<HEADER_LEN{return Err(PersistError::Truncated)}
    if input[..4]!=STATE_MAGIC{return Err(PersistError::BadMagic)}
    if input[4]!=STATE_VERSION{return Err(PersistError::UnsupportedVersion)}
    let total_len=u16::from_le_bytes([input[6],input[7]]) as usize;
    if total_len<HEADER_LEN||total_len>input.len()||total_len>MAX_STATE_SIZE{return Err(PersistError::InvalidCount)}
    let expected=u32::from_le_bytes([input[8],input[9],input[10],input[11]]);if checksum(&input[HEADER_LEN..total_len])!=expected{return Err(PersistError::ChecksumMismatch)}
    let mut pos=12usize;let vc=get_u16(input,&mut pos)? as usize;let mc=get_u16(input,&mut pos)? as usize;let sc=get_u16(input,&mut pos)? as usize;let fc=get_u16(input,&mut pos)? as usize;
    if vc>MAX_VOLUMES||mc>MAX_MOUNTS||sc>MAX_SNAPSHOTS||fc>MAX_PACKAGE_FILES{return Err(PersistError::InvalidCount)}
    let expected_payload=vc.checked_mul(VOLUME_REC_LEN).and_then(|v|v.checked_add(mc.checked_mul(MOUNT_REC_LEN)?)).and_then(|v|v.checked_add(sc.checked_mul(SNAPSHOT_REC_LEN)?)).and_then(|v|v.checked_add(fc.checked_mul(FILE_REC_LEN)?)).ok_or(PersistError::Overflow)?;
    if HEADER_LEN.checked_add(expected_payload).ok_or(PersistError::Overflow)?!=total_len{return Err(PersistError::InvalidCount)}
    let mut volumes=[None;MAX_VOLUMES];let mut mounts=[None;MAX_MOUNTS];let mut snapshots=[None;MAX_SNAPSHOTS];let mut files=[None;MAX_PACKAGE_FILES];let mut max_v=0u32;let mut max_m=0u32;let mut max_s=0u32;let mut max_f=0u32;
    for slot in volumes.iter_mut().take(vc){let id=get_u32(input,&mut pos)?;let ty=volume_type_from_u8(get_u8(input,&mut pos)?)?;let ro=get_u8(input,&mut pos)?!=0;let _=get_u16(input,&mut pos)?;let blocks=get_u64(input,&mut pos)?;let start=get_u64(input,&mut pos)?;if id==0||blocks==0{return Err(PersistError::InvalidRecord)}max_v=max_v.max(id);*slot=Some(StorageVolume{volume_id:id,volume_type:ty,block_count:blocks,start_lba:start,read_only:ro})}
    for slot in mounts.iter_mut().take(mc){let id=get_u32(input,&mut pos)?;let vid=get_u32(input,&mut pos)?;let path=get_u64(input,&mut pos)?;let active=get_u8(input,&mut pos)?!=0;if id==0||vid==0||!volumes.iter().flatten().any(|v|v.volume_id==vid){return Err(PersistError::InvalidRecord)}max_m=max_m.max(id);*slot=Some(MountEntry{mount_id:id,volume_id:vid,path_hash:path,active})}
    for slot in snapshots.iter_mut().take(sc){let id=get_u32(input,&mut pos)?;let vid=get_u32(input,&mut pos)?;let ts=get_u64(input,&mut pos)?;let delta=get_u32(input,&mut pos)?;let _=get_u32(input,&mut pos)?;if id==0||vid==0||!volumes.iter().flatten().any(|v|v.volume_id==vid){return Err(PersistError::InvalidRecord)}max_s=max_s.max(id);*slot=Some(SnapshotMetadata{snapshot_id:id,volume_id:vid,timestamp:ts,block_delta_count:delta})}
    for slot in files.iter_mut().take(fc){let id=get_u32(input,&mut pos)?;let vid=get_u32(input,&mut pos)?;let nh=get_u64(input,&mut pos)?;let size=get_u32(input,&mut pos)?;let installed=get_u8(input,&mut pos)?!=0;if id==0||vid==0||!volumes.iter().flatten().any(|v|v.volume_id==vid){return Err(PersistError::InvalidRecord)}max_f=max_f.max(id);*slot=Some(PackageFileRecord{file_id:id,volume_id:vid,name_hash:nh,size_bytes:size,is_installed_package:installed})}
    *storage=StorageDaemon{volumes,mounts,snapshots,files,cache:[None;CACHE_BLOCKS],volume_counter:max_v.saturating_add(1).max(1),mount_counter:max_m.saturating_add(1).max(1),snapshot_counter:max_s.saturating_add(1).max(1),file_counter:max_f.saturating_add(1).max(1),self_healed_events:0};Ok(())
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn state_round_trip_restores_metadata_and_counters(){let mut s=StorageDaemon::new();let v=s.register_volume(VolumeType::AweFsVolume,4096,64,false).unwrap();s.mount_volume(v,0x55AA).unwrap();s.create_snapshot(v,123456).unwrap();s.store_package_file(v,0xABCDEF,8192).unwrap();let mut b=[0u8;MAX_STATE_SIZE];let len=export_state(&s,&mut b).unwrap();let mut r=StorageDaemon::new();import_state(&mut r,&b[..len]).unwrap();assert_eq!(r.volumes[0].unwrap().block_count,4096);assert_eq!(r.mounts[0].unwrap().path_hash,0x55AA);assert_eq!(r.snapshots[0].unwrap().timestamp,123456);assert_eq!(r.files[0].unwrap().size_bytes,8192);assert_eq!(r.register_volume(VolumeType::Ramdisk,1,1,true).unwrap(),2);assert_eq!(r.mount_volume(2,3).unwrap(),2)}
 #[test]fn tampering_is_rejected_by_checksum(){let s=StorageDaemon::new();let mut b=[0u8;MAX_STATE_SIZE];let len=export_state(&s,&mut b).unwrap();b[len-1]^=1;let mut r=StorageDaemon::new();assert_eq!(import_state(&mut r,&b[..len]),Err(PersistError::ChecksumMismatch))}
 #[test]fn header_counts_survive_checksum_storage(){let mut s=StorageDaemon::new();let v=s.register_volume(VolumeType::VirtualDisk,32,7,false).unwrap();s.mount_volume(v,9).unwrap();let mut b=[0u8;MAX_STATE_SIZE];let len=export_state(&s,&mut b).unwrap();assert!(len>=HEADER_LEN);let mut r=StorageDaemon::new();import_state(&mut r,&b[..len]).unwrap();assert_eq!(r.volumes[0].unwrap().volume_id,1);assert!(r.mounts[0].is_some())}
}
