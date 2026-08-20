//! AWEOS User-Space Storage Service (`storaged`)
//!
//! Manages block device volumes, GPT partitioning, VFS mount tables,
//! LRU block cache, and filesystem snapshots.

#![no_std]

pub const MAX_VOLUMES: usize = 16;
pub const MAX_MOUNTS: usize = 16;
pub const MAX_SNAPSHOTS: usize = 8;
pub const BLOCK_SIZE: usize = 512;
pub const CACHE_BLOCKS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeType {
    Ramdisk,
    GptPartition,
    VirtualDisk,
}

#[derive(Debug, Clone, Copy)]
pub struct StorageVolume {
    pub volume_id: u32,
    pub volume_type: VolumeType,
    pub block_count: u64,
    pub start_lba: u64,
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MountEntry {
    pub mount_id: u32,
    pub volume_id: u32,
    pub path_hash: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SnapshotMetadata {
    pub snapshot_id: u32,
    pub volume_id: u32,
    pub timestamp: u64,
    pub block_delta_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CachedBlock {
    pub volume_id: u32,
    pub lba: u64,
    pub dirty: bool,
    pub data: [u8; BLOCK_SIZE],
}

/// Storage Daemon Supervisor Instance.
#[derive(Debug)]
pub struct StorageDaemon {
    volumes: [Option<StorageVolume>; MAX_VOLUMES],
    mounts: [Option<MountEntry>; MAX_MOUNTS],
    snapshots: [Option<SnapshotMetadata>; MAX_SNAPSHOTS],
    cache: [Option<CachedBlock>; CACHE_BLOCKS],
    volume_counter: u32,
    mount_counter: u32,
    snapshot_counter: u32,
}

impl StorageDaemon {
    pub const fn new() -> Self {
        Self {
            volumes: [None; MAX_VOLUMES],
            mounts: [None; MAX_MOUNTS],
            snapshots: [None; MAX_SNAPSHOTS],
            cache: [None; CACHE_BLOCKS],
            volume_counter: 1,
            mount_counter: 1,
            snapshot_counter: 1,
        }
    }

    pub fn register_volume(
        &mut self,
        vol_type: VolumeType,
        block_count: u64,
        start_lba: u64,
        read_only: bool,
    ) -> Result<u32, &'static str> {
        let vid = self.volume_counter;
        for slot in self.volumes.iter_mut() {
            if slot.is_none() {
                *slot = Some(StorageVolume {
                    volume_id: vid,
                    volume_type: vol_type,
                    block_count,
                    start_lba,
                    read_only,
                });
                self.volume_counter += 1;
                return Ok(vid);
            }
        }
        Err("Volume table full")
    }

    pub fn mount_volume(&mut self, volume_id: u32, path_hash: u64) -> Result<u32, &'static str> {
        let mut found = false;
        for v in self.volumes.iter().flatten() {
            if v.volume_id == volume_id {
                found = true;
                break;
            }
        }
        if !found {
            return Err("Volume ID not found");
        }

        let mid = self.mount_counter;
        for slot in self.mounts.iter_mut() {
            if slot.is_none() {
                *slot = Some(MountEntry {
                    mount_id: mid,
                    volume_id,
                    path_hash,
                    active: true,
                });
                self.mount_counter += 1;
                return Ok(mid);
            }
        }
        Err("Mount table full")
    }

    pub fn create_snapshot(&mut self, volume_id: u32, timestamp: u64) -> Result<u32, &'static str> {
        let sid = self.snapshot_counter;
        for slot in self.snapshots.iter_mut() {
            if slot.is_none() {
                *slot = Some(SnapshotMetadata {
                    snapshot_id: sid,
                    volume_id,
                    timestamp,
                    block_delta_count: 0,
                });
                self.snapshot_counter += 1;
                return Ok(sid);
            }
        }
        Err("Snapshot table full")
    }

    pub fn cache_read(&self, volume_id: u32, lba: u64) -> Option<&[u8; BLOCK_SIZE]> {
        for block in self.cache.iter().flatten() {
            if block.volume_id == volume_id && block.lba == lba {
                return Some(&block.data);
            }
        }
        None
    }

    pub fn cache_write(
        &mut self,
        volume_id: u32,
        lba: u64,
        data: &[u8; BLOCK_SIZE],
    ) -> Result<(), &'static str> {
        // Update existing cache block if present
        for block in self.cache.iter_mut().flatten() {
            if block.volume_id == volume_id && block.lba == lba {
                block.data = *data;
                block.dirty = true;
                return Ok(());
            }
        }
        // Insert into free slot
        for slot in self.cache.iter_mut() {
            if slot.is_none() {
                *slot = Some(CachedBlock {
                    volume_id,
                    lba,
                    dirty: true,
                    data: *data,
                });
                return Ok(());
            }
        }
        Err("Block cache full")
    }
}

impl Default for StorageDaemon {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storaged_lifecycle() {
        let mut storaged = StorageDaemon::new();
        let vid = storaged
            .register_volume(VolumeType::Ramdisk, 2048, 0, false)
            .unwrap();
        assert_eq!(vid, 1);

        let mid = storaged.mount_volume(vid, 0x1234_5678).unwrap();
        assert_eq!(mid, 1);

        let snap_id = storaged.create_snapshot(vid, 1000).unwrap();
        assert_eq!(snap_id, 1);

        let block_data = [0xAB; BLOCK_SIZE];
        storaged.cache_write(vid, 42, &block_data).unwrap();

        let cached = storaged
            .cache_read(vid, 42)
            .expect("Should hit block cache");
        assert_eq!(cached[0], 0xAB);
    }
}
