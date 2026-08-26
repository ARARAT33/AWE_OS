#![no_std]

use super::{PackageFileRecord, SnapshotMetadata, StorageDaemon, StorageVolume, VolumeType, MAX_MOUNTS, MAX_PACKAGE_FILES, MAX_SNAPSHOTS, MAX_VOLUMES};

pub const STATE_MAGIC: [u8; 4] = *b"AWSP";
pub const STATE_VERSION: u8 = 1;
pub const MAX_STATE_SIZE: usize = 4096;
const HEADER_LEN: usize = 16;
const VOLUME_REC_LEN: usize = 20;
const MOUNT_REC_LEN: usize = 17;
const SNAPSHOT_REC_LEN: usize = 24;
const FILE_REC_LEN: usize = 25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistError {
    BufferTooSmall,
    Truncated,
    BadMagic,
    UnsupportedVersion,
    ChecksumMismatch,
    InvalidRecord,
    InvalidCount,
    Overflow,
}

fn checksum(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5u32;
    for &byte in bytes {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn put_u16(out: &mut [u8], pos: &mut usize, value: u16) -> Result<(), PersistError> {
    let end = pos.checked_add(2).ok_or(PersistError::Overflow)?;
    if end > out.len() { return Err(PersistError::BufferTooSmall); }
    out[*pos..end].copy_from_slice(&value.to_le_bytes());
    *pos = end;
    Ok(())
}
fn put_u32(out: &mut [u8], pos: &mut usize, value: u32) -> Result<(), PersistError> {
    let end = pos.checked_add(4).ok_or(PersistError::Overflow)?;
    if end > out.len() { return Err(PersistError::BufferTooSmall); }
    out[*pos..end].copy_from_slice(&value.to_le_bytes());
    *pos = end;
    Ok(())
}
fn put_u64(out: &mut [u8], pos: &mut usize, value: u64) -> Result<(), PersistError> {
    let end = pos.checked_add(8).ok_or(PersistError::Overflow)?;
    if end > out.len() { return Err(PersistError::BufferTooSmall); }
    out[*pos..end].copy_from_slice(&value.to_le_bytes());
    *pos = end;
    Ok(())
}
fn put_u8(out: &mut [u8], pos: &mut usize, value: u8) -> Result<(), PersistError> {
    if *pos >= out.len() { return Err(PersistError::BufferTooSmall); }
    out[*pos] = value;
    *pos += 1;
    Ok(())
}
fn get_u16(input: &[u8], pos: &mut usize) -> Result<u16, PersistError> {
    let end = pos.checked_add(2).ok_or(PersistError::Overflow)?;
    if end > input.len() { return Err(PersistError::Truncated); }
    let value = u16::from_le_bytes([input[*pos], input[*pos + 1]]);
    *pos = end;
    Ok(value)
}
fn get_u32(input: &[u8], pos: &mut usize) -> Result<u32, PersistError> {
    let end = pos.checked_add(4).ok_or(PersistError::Overflow)?;
    if end > input.len() { return Err(PersistError::Truncated); }
    let value = u32::from_le_bytes([input[*pos], input[*pos + 1], input[*pos + 2], input[*pos + 3]]);
    *pos = end;
    Ok(value)
}
fn get_u64(input: &[u8], pos: &mut usize) -> Result<u64, PersistError> {
    let end = pos.checked_add(8).ok_or(PersistError::Overflow)?;
    if end > input.len() { return Err(PersistError::Truncated); }
    let value = u64::from_le_bytes([
        input[*pos], input[*pos + 1], input[*pos + 2], input[*pos + 3],
        input[*pos + 4], input[*pos + 5], input[*pos + 6], input[*pos + 7],
    ]);
    *pos = end;
    Ok(value)
}
fn get_u8(input: &[u8], pos: &mut usize) -> Result<u8, PersistError> {
    if *pos >= input.len() { return Err(PersistError::Truncated); }
    let value = input[*pos];
    *pos += 1;
    Ok(value)
}

fn volume_type_to_u8(value: VolumeType) -> u8 {
    match value { VolumeType::Ramdisk => 0, VolumeType::GptPartition => 1, VolumeType::VirtualDisk => 2, VolumeType::AweFsVolume => 3 }
}
fn volume_type_from_u8(value: u8) -> Result<VolumeType, PersistError> {
    match value { 0 => Ok(VolumeType::Ramdisk), 1 => Ok(VolumeType::GptPartition), 2 => Ok(VolumeType::VirtualDisk), 3 => Ok(VolumeType::AweFsVolume), _ => Err(PersistError::InvalidRecord) }
}

pub fn export_state(storage: &StorageDaemon, out: &mut [u8]) -> Result<usize, PersistError> {
    if out.len() < HEADER_LEN { return Err(PersistError::BufferTooSmall); }
    let volume_count = storage.volumes.iter().filter(|v| v.is_some()).count();
    let mount_count = storage.mounts.iter().filter(|v| v.is_some()).count();
    let snapshot_count = storage.snapshots.iter().filter(|v| v.is_some()).count();
    let file_count = storage.files.iter().filter(|v| v.is_some()).count();
    if volume_count > MAX_VOLUMES || mount_count > MAX_MOUNTS || snapshot_count > MAX_SNAPSHOTS || file_count > MAX_PACKAGE_FILES { return Err(PersistError::InvalidCount); }

    let payload_len = volume_count.checked_mul(VOLUME_REC_LEN).and_then(|v| v.checked_add(mount_count.checked_mul(MOUNT_REC_LEN)?)).and_then(|v| v.checked_add(snapshot_count.checked_mul(SNAPSHOT_REC_LEN)?)).and_then(|v| v.checked_add(file_count.checked_mul(FILE_REC_LEN)?)).ok_or(PersistError::Overflow)?;
    let total_len = HEADER_LEN.checked_add(payload_len).ok_or(PersistError::Overflow)?;
    if total_len > out.len() || total_len > MAX_STATE_SIZE { return Err(PersistError::BufferTooSmall); }
    out[..total_len].fill(0);
    out[..4].copy_from_slice(&STATE_MAGIC);
    out[4] = STATE_VERSION;
    let mut pos = 8usize;
    put_u16(out, &mut pos, volume_count as u16)?;
    put_u16(out, &mut pos, mount_count as u16)?;
    put_u16(out, &mut pos, snapshot_count as u16)?;
    put_u16(out, &mut pos, file_count as u16)?;

    for rec in storage.volumes.iter().flatten() {
        put_u32(out, &mut pos, rec.volume_id)?; put_u8(out, &mut pos, volume_type_to_u8(rec.volume_type))?; put_u8(out, &mut pos, u8::from(rec.read_only))?; put_u16(out, &mut pos, 0)?; put_u64(out, &mut pos, rec.block_count)?; put_u64(out, &mut pos, rec.start_lba)?;
    }
    for rec in storage.mounts.iter().flatten() {
        put_u32(out, &mut pos, rec.mount_id)?; put_u32(out, &mut pos, rec.volume_id)?; put_u64(out, &mut pos, rec.path_hash)?; put_u8(out, &mut pos, u8::from(rec.active))?;
    }
    for rec in storage.snapshots.iter().flatten() {
        put_u32(out, &mut pos, rec.snapshot_id)?; put_u32(out, &mut pos, rec.volume_id)?; put_u64(out, &mut pos, rec.timestamp)?; put_u32(out, &mut pos, rec.block_delta_count)?; put_u32(out, &mut pos, 0)?;
    }
    for rec in storage.files.iter().flatten() {
        put_u32(out, &mut pos, rec.file_id)?; put_u32(out, &mut pos, rec.volume_id)?; put_u64(out, &mut pos, rec.name_hash)?; put_u32(out, &mut pos, rec.size_bytes)?; put_u8(out, &mut pos, u8::from(rec.is_installed_package))?;
    }
    let sum = checksum(&out[HEADER_LEN..total_len]);
    out[0..4].copy_from_slice(&STATE_MAGIC);
    out[5] = 0;
    out[6..8].copy_from_slice(&(total_len as u16).to_le_bytes());
    out[8..12].copy_from_slice(&sum.to_le_bytes());
    Ok(total_len)
}

pub fn import_state(storage: &mut StorageDaemon, input: &[u8]) -> Result<(), PersistError> {
    if input.len() < HEADER_LEN { return Err(PersistError::Truncated); }
    if input[..4] != STATE_MAGIC { return Err(PersistError::BadMagic); }
    if input[4] != STATE_VERSION { return Err(PersistError::UnsupportedVersion); }
    let total_len = u16::from_le_bytes([input[6], input[7]]) as usize;
    if total_len < HEADER_LEN || total_len > input.len() || total_len > MAX_STATE_SIZE { return Err(PersistError::InvalidCount); }
    let expected_sum = u32::from_le_bytes([input[8], input[9], input[10], input[11]]);
    if checksum(&input[HEADER_LEN..total_len]) != expected_sum { return Err(PersistError::ChecksumMismatch); }
    let mut pos = 12usize;
    pos += 4;
    let volume_count = get_u16(input, &mut pos)? as usize;
    let mount_count = get_u16(input, &mut pos)? as usize;
    let snapshot_count = get_u16(input, &mut pos)? as usize;
    let file_count = get_u16(input, &mut pos)? as usize;
    if volume_count > MAX_VOLUMES || mount_count > MAX_MOUNTS || snapshot_count > MAX_SNAPSHOTS || file_count > MAX_PACKAGE_FILES { return Err(PersistError::InvalidCount); }
    let expected_payload = volume_count.checked_mul(VOLUME_REC_LEN).and_then(|v| v.checked_add(mount_count.checked_mul(MOUNT_REC_LEN)?)).and_then(|v| v.checked_add(snapshot_count.checked_mul(SNAPSHOT_REC_LEN)?)).and_then(|v| v.checked_add(file_count.checked_mul(FILE_REC_LEN)?)).ok_or(PersistError::Overflow)?;
    if HEADER_LEN.checked_add(expected_payload).ok_or(PersistError::Overflow)? != total_len { return Err(PersistError::InvalidCount); }

    let mut volumes = [None; MAX_VOLUMES];
    let mut mounts = [None; MAX_MOUNTS];
    let mut snapshots = [None; MAX_SNAPSHOTS];
    let mut files = [None; MAX_PACKAGE_FILES];
    let mut max_volume_id = 0u32; let mut max_mount_id = 0u32; let mut max_snapshot_id = 0u32; let mut max_file_id = 0u32;

    for slot in volumes.iter_mut().take(volume_count) {
        let id=get_u32(input,&mut pos)?; let ty=volume_type_from_u8(get_u8(input,&mut pos)?)?; let read_only=get_u8(input,&mut pos)? != 0; let _=get_u16(input,&mut pos)?; let blocks=get_u64(input,&mut pos)?; let start=get_u64(input,&mut pos)?;
        if id==0 || blocks==0 { return Err(PersistError::InvalidRecord); } max_volume_id=max_volume_id.max(id); *slot=Some(StorageVolume{volume_id:id,volume_type:ty,block_count:blocks,start_lba:start,read_only});
    }
    for slot in mounts.iter_mut().take(mount_count) {
        let id=get_u32(input,&mut pos)?; let volume_id=get_u32(input,&mut pos)?; let path_hash=get_u64(input,&mut pos)?; let active=get_u8(input,&mut pos)? != 0;
        if id==0 || volume_id==0 { return Err(PersistError::InvalidRecord); } max_mount_id=max_mount_id.max(id); slot.replace(super::MountEntry{mount_id:id,volume_id,path_hash,active});
    }
    for slot in snapshots.iter_mut().take(snapshot_count) {
        let id=get_u32(input,&mut pos)?; let volume_id=get_u32(input,&mut pos)?; let timestamp=get_u64(input,&mut pos)?; let delta=get_u32(input,&mut pos)?; let _=get_u32(input,&mut pos)?;
        if id==0 || volume_id==0 { return Err(PersistError::InvalidRecord); } max_snapshot_id=max_snapshot_id.max(id); slot.replace(SnapshotMetadata{snapshot_id:id,volume_id,timestamp,block_delta_count:delta});
    }
    for slot in files.iter_mut().take(file_count) {
        let id=get_u32(input,&mut pos)?; let volume_id=get_u32(input,&mut pos)?; let name_hash=get_u64(input,&mut pos)?; let size_bytes=get_u32(input,&mut pos)?; let installed=get_u8(input,&mut pos)? != 0;
        if id==0 || volume_id==0 { return Err(PersistError::InvalidRecord); } max_file_id=max_file_id.max(id); slot.replace(PackageFileRecord{file_id:id,volume_id,name_hash,size_bytes,is_installed_package:installed});
    }

    *storage = StorageDaemon {
        volumes, mounts, snapshots, files,
        cache: [None; super::CACHE_BLOCKS],
        volume_counter: max_volume_id.saturating_add(1).max(1),
        mount_counter: max_mount_id.saturating_add(1).max(1),
        snapshot_counter: max_snapshot_id.saturating_add(1).max(1),
        file_counter: max_file_id.saturating_add(1).max(1),
        self_healed_events: 0,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trip_restores_metadata_and_counters() {
        let mut source = StorageDaemon::new();
        let volume = source.register_volume(VolumeType::AweFsVolume, 4096, 64, false).unwrap();
        source.mount_volume(volume, 0x55AA).unwrap();
        source.create_snapshot(volume, 123456).unwrap();
        source.store_package_file(volume, 0xABCDEF, 8192).unwrap();
        let mut bytes = [0u8; MAX_STATE_SIZE];
        let len = export_state(&source, &mut bytes).unwrap();

        let mut restored = StorageDaemon::new();
        import_state(&mut restored, &bytes[..len]).unwrap();
        assert_eq!(restored.volumes[0].unwrap().block_count, 4096);
        assert_eq!(restored.mounts[0].unwrap().path_hash, 0x55AA);
        assert_eq!(restored.snapshots[0].unwrap().timestamp, 123456);
        assert_eq!(restored.files[0].unwrap().size_bytes, 8192);
        assert_eq!(restored.register_volume(VolumeType::Ramdisk, 1, 1, true).unwrap(), 2);
        assert_eq!(restored.mount_volume(2, 3).unwrap(), 2);
    }

    #[test]
    fn tampering_is_rejected_by_checksum() {
        let source = StorageDaemon::new();
        let mut bytes = [0u8; MAX_STATE_SIZE];
        let len = export_state(&source, &mut bytes).unwrap();
        bytes[len - 1] ^= 1;
        let mut restored = StorageDaemon::new();
        assert_eq!(import_state(&mut restored, &bytes[..len]), Err(PersistError::ChecksumMismatch));
    }
}
