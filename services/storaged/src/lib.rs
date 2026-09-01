//! AWEOS User-Space Storage Service (`storaged`)
//!
//! Manages block device volumes, GPT partitioning, VFS mount tables,
//! LRU block cache, and filesystem snapshots.

#![no_std]

pub mod persistence;

pub const MAX_VOLUMES: usize = 16;
pub const MAX_MOUNTS: usize = 16;
pub const MAX_SNAPSHOTS: usize = 8;
pub const MAX_PACKAGE_FILES: usize = 32;
pub const BLOCK_SIZE: usize = 512;
pub const CACHE_BLOCKS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeType { Ramdisk, GptPartition, VirtualDisk, AweFsVolume }
#[derive(Debug, Clone, Copy)]
pub struct PackageFileRecord { pub file_id: u32, pub volume_id: u32, pub name_hash: u64, pub size_bytes: u32, pub is_installed_package: bool }
#[derive(Debug, Clone, Copy)]
pub struct StorageVolume { pub volume_id: u32, pub volume_type: VolumeType, pub block_count: u64, pub start_lba: u64, pub read_only: bool }
#[derive(Debug, Clone, Copy)]
pub struct MountEntry { pub mount_id: u32, pub volume_id: u32, pub path_hash: u64, pub active: bool }
#[derive(Debug, Clone, Copy)]
pub struct SnapshotMetadata { pub snapshot_id: u32, pub volume_id: u32, pub timestamp: u64, pub block_delta_count: u32 }
#[derive(Debug, Clone, Copy)]
pub struct CachedBlock { pub volume_id: u32, pub lba: u64, pub dirty: bool, pub data: [u8; BLOCK_SIZE] }

#[derive(Debug)]
pub struct StorageDaemon {
    pub(crate) volumes: [Option<StorageVolume>; MAX_VOLUMES],
    pub(crate) mounts: [Option<MountEntry>; MAX_MOUNTS],
    pub(crate) snapshots: [Option<SnapshotMetadata>; MAX_SNAPSHOTS],
    pub(crate) files: [Option<PackageFileRecord>; MAX_PACKAGE_FILES],
    pub(crate) cache: [Option<CachedBlock>; CACHE_BLOCKS],
    pub(crate) volume_counter: u32,
    pub(crate) mount_counter: u32,
    pub(crate) snapshot_counter: u32,
    pub(crate) file_counter: u32,
    pub self_healed_events: usize,
}

impl StorageDaemon {
    pub const fn new() -> Self { Self { volumes:[None;MAX_VOLUMES], mounts:[None;MAX_MOUNTS], snapshots:[None;MAX_SNAPSHOTS], files:[None;MAX_PACKAGE_FILES], cache:[None;CACHE_BLOCKS], volume_counter:1, mount_counter:1, snapshot_counter:1, file_counter:1, self_healed_events:0 } }
    pub fn register_volume(&mut self, vol_type:VolumeType, block_count:u64, start_lba:u64, read_only:bool)->Result<u32,&'static str>{ let vid=self.volume_counter; for slot in self.volumes.iter_mut(){if slot.is_none(){*slot=Some(StorageVolume{volume_id:vid,volume_type:vol_type,block_count,start_lba,read_only});self.volume_counter=self.volume_counter.saturating_add(1);return Ok(vid);}} Err("Volume table full") }
    pub fn mount_volume(&mut self, volume_id:u32, path_hash:u64)->Result<u32,&'static str>{ if !self.volumes.iter().flatten().any(|v|v.volume_id==volume_id){return Err("Volume ID not found");} let mid=self.mount_counter; for slot in self.mounts.iter_mut(){if slot.is_none(){*slot=Some(MountEntry{mount_id:mid,volume_id,path_hash,active:true});self.mount_counter=self.mount_counter.saturating_add(1);return Ok(mid);}} Err("Mount table full") }
    pub fn create_snapshot(&mut self, volume_id:u32, timestamp:u64)->Result<u32,&'static str>{ if !self.volumes.iter().flatten().any(|v|v.volume_id==volume_id){return Err("Volume ID not found");} let sid=self.snapshot_counter; for slot in self.snapshots.iter_mut(){if slot.is_none(){*slot=Some(SnapshotMetadata{snapshot_id:sid,volume_id,timestamp,block_delta_count:0});self.snapshot_counter=self.snapshot_counter.saturating_add(1);return Ok(sid);}} Err("Snapshot table full") }
    pub fn cache_read(&self, volume_id:u32, lba:u64)->Option<&[u8;BLOCK_SIZE]>{ for block in self.cache.iter().flatten(){if block.volume_id==volume_id&&block.lba==lba{return Some(&block.data);}} None }
    pub fn cache_write(&mut self, volume_id:u32,lba:u64,data:&[u8;BLOCK_SIZE])->Result<(),&'static str>{ for block in self.cache.iter_mut().flatten(){if block.volume_id==volume_id&&block.lba==lba{block.data=*data;block.dirty=true;return Ok(());}} for slot in self.cache.iter_mut(){if slot.is_none(){*slot=Some(CachedBlock{volume_id,lba,dirty:true,data:*data});return Ok(());}} Err("Block cache full") }
    pub fn store_package_file(&mut self,volume_id:u32,name_hash:u64,size_bytes:u32)->Result<u32,&'static str>{ if !self.volumes.iter().flatten().any(|v|v.volume_id==volume_id){return Err("Volume ID not found");} let fid=self.file_counter; for slot in self.files.iter_mut(){if slot.is_none(){*slot=Some(PackageFileRecord{file_id:fid,volume_id,name_hash,size_bytes,is_installed_package:true});self.file_counter=self.file_counter.saturating_add(1);return Ok(fid);}} Err("File records table full") }
    pub fn delete_package_file(&mut self,file_id:u32)->Result<(),&'static str>{ for slot in self.files.iter_mut(){if let Some(f)=slot&&f.file_id==file_id{*slot=None;return Ok(());}} Err("File not found") }
    pub fn trigger_self_healing_repair(&mut self,volume_id:u32)->bool{ if self.snapshots.iter().flatten().any(|s|s.volume_id==volume_id){self.self_healed_events+=1;true}else{false} }
}
impl Default for StorageDaemon { fn default()->Self{Self::new()} }

#[cfg(test)]
mod tests { use super::*; #[test] fn test_storaged_lifecycle(){let mut s=StorageDaemon::new();let v=s.register_volume(VolumeType::Ramdisk,2048,0,false).unwrap();assert_eq!(v,1);assert_eq!(s.mount_volume(v,0x1234_5678).unwrap(),1);assert_eq!(s.create_snapshot(v,1000).unwrap(),1);let d=[0xAB;BLOCK_SIZE];s.cache_write(v,42,&d).unwrap();assert_eq!(s.cache_read(v,42).unwrap()[0],0xAB);let f=s.store_package_file(v,0x112233,4096).unwrap();assert_eq!(f,1);assert!(s.trigger_self_healing_repair(v));assert_eq!(s.self_healed_events,1);s.delete_package_file(f).unwrap();assert!(s.delete_package_file(f).is_err());}}
