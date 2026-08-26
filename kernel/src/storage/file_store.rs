#![no_std]

use super::{BlockDevice, StorageError, BLOCK_SIZE};

pub const MAX_FILES: usize = 64;
pub const MAX_FILE_BLOCKS: usize = 32;
pub const MAX_FILE_SIZE: usize = MAX_FILE_BLOCKS * BLOCK_SIZE;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileStoreError { Full, NotFound, InvalidName, InvalidOffset, TooLarge, NoSpace, Storage(StorageError) }
impl From<StorageError> for FileStoreError { fn from(v: StorageError) -> Self { Self::Storage(v) } }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileRecord { pub id: u32, pub size: u64, pub first_block: u64, pub blocks: u16, pub generation: u64 }

#[derive(Clone, Copy)]
struct Slot { record: Option<FileRecord>, name: [u8; 63], name_len: u8 }
impl Slot { const fn empty() -> Self { Self { record: None, name: [0;63], name_len: 0 } } }

#[derive(Clone, Copy)]
pub struct FileStore<const N: usize = MAX_FILES> {
    slots: [Slot; N],
    next_id: u32,
    next_block: u64,
}
impl<const N: usize> FileStore<N> {
    pub const fn new(start_block: u64) -> Self { Self { slots: [Slot::empty(); N], next_id: 1, next_block: start_block } }
    pub fn create(&mut self, name: &[u8]) -> Result<FileRecord, FileStoreError> {
        if name.is_empty() || name.len() > 63 || name.iter().any(|b| *b == 0 || *b == b'/') { return Err(FileStoreError::InvalidName); }
        if self.find(name).is_some() { return Err(FileStoreError::InvalidName); }
        let slot = self.slots.iter().position(|s| s.record.is_none()).ok_or(FileStoreError::Full)?;
        let record = FileRecord { id: self.next_id, size: 0, first_block: self.next_block, blocks: 0, generation: 1 };
        self.next_id = self.next_id.checked_add(1).ok_or(FileStoreError::Full)?;
        self.slots[slot].record = Some(record); self.slots[slot].name[..name.len()].copy_from_slice(name); self.slots[slot].name_len = name.len() as u8;
        Ok(record)
    }
    pub fn write_at<D: BlockDevice>(&mut self, device: &mut D, id: u32, offset: usize, data: &[u8]) -> Result<usize, FileStoreError> {
        let index = self.find_id(id).ok_or(FileStoreError::NotFound)?; let record = self.slots[index].record.unwrap();
        let end = offset.checked_add(data.len()).ok_or(FileStoreError::TooLarge)?;
        if end > MAX_FILE_SIZE { return Err(FileStoreError::TooLarge); } if data.is_empty() { return Ok(0); }
        let required_blocks = end.div_ceil(BLOCK_SIZE); if required_blocks > MAX_FILE_BLOCKS { return Err(FileStoreError::TooLarge); }
        if required_blocks > record.blocks as usize { let additional = required_blocks - record.blocks as usize; self.next_block = self.next_block.checked_add(additional as u64).ok_or(FileStoreError::NoSpace)?; self.slots[index].record.as_mut().unwrap().blocks = required_blocks as u16; }
        let first = record.first_block; let mut consumed = 0usize;
        while consumed < data.len() {
            let absolute = offset + consumed; let block = first + (absolute / BLOCK_SIZE) as u64; let in_block = absolute % BLOCK_SIZE; let take = core::cmp::min(BLOCK_SIZE - in_block, data.len() - consumed); let mut buf = [0u8; BLOCK_SIZE];
            device.read_block(block, &mut buf)?; buf[in_block..in_block + take].copy_from_slice(&data[consumed..consumed + take]); device.write_block(block, &buf)?; consumed += take;
        }
        let current = self.slots[index].record.as_mut().unwrap(); if end as u64 > current.size { current.size = end as u64; } current.generation = current.generation.saturating_add(1); Ok(data.len())
    }
    pub fn read_at<D: BlockDevice>(&self, device: &mut D, id: u32, offset: usize, out: &mut [u8]) -> Result<usize, FileStoreError> {
        let index = self.find_id(id).ok_or(FileStoreError::NotFound)?; let record = self.slots[index].record.unwrap(); if offset > record.size as usize { return Err(FileStoreError::InvalidOffset); }
        let count = core::cmp::min(record.size as usize - offset, out.len()); let mut consumed = 0usize;
        while consumed < count { let absolute = offset + consumed; let block = record.first_block + (absolute / BLOCK_SIZE) as u64; let in_block = absolute % BLOCK_SIZE; let take = core::cmp::min(BLOCK_SIZE - in_block, count - consumed); let mut buf = [0u8; BLOCK_SIZE]; device.read_block(block, &mut buf)?; out[consumed..consumed+take].copy_from_slice(&buf[in_block..in_block+take]); consumed += take; }
        Ok(count)
    }
    pub fn truncate(&mut self, id: u32, size: usize) -> Result<(), FileStoreError> { if size > MAX_FILE_SIZE { return Err(FileStoreError::TooLarge); } let index = self.find_id(id).ok_or(FileStoreError::NotFound)?; let r = self.slots[index].record.as_mut().unwrap(); r.size=size as u64; r.blocks=size.div_ceil(BLOCK_SIZE) as u16; r.generation=r.generation.saturating_add(1); Ok(()) }
    pub fn delete(&mut self, id: u32) -> Result<(), FileStoreError> { let index=self.find_id(id).ok_or(FileStoreError::NotFound)?; self.slots[index]=Slot::empty(); Ok(()) }
    pub fn lookup(&self, name: &[u8]) -> Option<FileRecord> { self.find(name).and_then(|i| self.slots[i].record) }
    pub fn record(&self, id: u32) -> Option<FileRecord> { self.find_id(id).and_then(|i| self.slots[i].record) }
    fn find(&self, name: &[u8]) -> Option<usize> { self.slots.iter().position(|s| s.record.is_some() && s.name_len as usize == name.len() && &s.name[..name.len()] == name) }
    fn find_id(&self, id: u32) -> Option<usize> { self.slots.iter().position(|s| s.record.map(|r| r.id) == Some(id)) }
}
impl<const N: usize> Default for FileStore<N> { fn default() -> Self { Self::new(8) } }

#[cfg(test)]
mod tests {
    use super::*; use crate::storage::RamBlockDevice;
    #[test] fn block_backed_round_trip_supports_seek() { let mut disk=RamBlockDevice::default(); let mut fs=FileStore::<4>::new(8); let file=fs.create(b"persist.bin").unwrap(); fs.write_at(&mut disk,file.id,3,b"AWEOS").unwrap(); let mut out=[0u8;8]; assert_eq!(fs.read_at(&mut disk,file.id,3,&mut out).unwrap(),5); assert_eq!(&out[..5],b"AWEOS"); assert_eq!(fs.record(file.id).unwrap().size,8); }
    #[test] fn rejects_out_of_bounds_file_size() { let mut disk=RamBlockDevice::default(); let mut fs=FileStore::<1>::new(8); let file=fs.create(b"x").unwrap(); assert_eq!(fs.write_at(&mut disk,file.id,MAX_FILE_SIZE,b"x"),Err(FileStoreError::TooLarge)); }
}
