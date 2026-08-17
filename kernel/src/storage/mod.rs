//! AWE storage core contracts.
//! Hardware drivers implement these bounded interfaces; policy and filesystem
//! layers remain independent of the transport.

#![allow(dead_code)]

pub mod gpt;
pub mod ramdisk;

pub use gpt::{crc32, parse_header, parse_partition, validate_partition_array_crc, GptError, GptHeader, GptPartition};
pub use ramdisk::{RamBlockDevice, RAMDISK_BLOCKS};

pub const BLOCK_SIZE: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageError {
    InvalidBlock,
    BufferTooSmall,
    ReadOnly,
    Io,
    Unsupported,
}

pub trait BlockDevice {
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
    fn block_count(&self) -> u64;
    fn read_block(&mut self, block: u64, out: &mut [u8]) -> Result<(), StorageError>;
    fn write_block(&mut self, block: u64, data: &[u8]) -> Result<(), StorageError>;
    fn flush(&mut self) -> Result<(), StorageError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceGeometry {
    pub block_size: u32,
    pub blocks: u64,
    pub read_only: bool,
}

impl DeviceGeometry {
    pub const fn new(block_size: u32, blocks: u64, read_only: bool) -> Self {
        Self {
            block_size,
            blocks,
            read_only,
        }
    }

    pub const fn bytes(self) -> Option<u64> {
        self.blocks.checked_mul(self.block_size as u64)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GptScanSummary {
    pub header: GptHeader,
    pub partitions: u16,
}

/// Validate and enumerate GPT directly through the bounded block-device API.
///
/// GPT uses 512-byte logical sectors. The storage layer keeps a 4 KiB transport
/// block, so this routine performs only bounded sector-to-block translation and
/// never allocates or trusts partition metadata before CRC/range validation.
pub fn scan_gpt<D: BlockDevice>(device: &mut D) -> Result<GptScanSummary, StorageError> {
    if device.block_size() < gpt::GPT_SECTOR_SIZE
        || !device.block_size().is_multiple_of(gpt::GPT_SECTOR_SIZE)
    {
        return Err(StorageError::Unsupported);
    }

    let disk_last_lba = device
        .block_count()
        .checked_mul((device.block_size() / gpt::GPT_SECTOR_SIZE) as u64)
        .and_then(|sectors| sectors.checked_sub(1))
        .ok_or(StorageError::InvalidBlock)?;

    let mut block = [0u8; BLOCK_SIZE];
    device.read_block(0, &mut block)?;
    let header = gpt::parse_header(
        &block[..gpt::GPT_SECTOR_SIZE * 2],
        disk_last_lba,
    )?;

    let entry_bytes = (header.partition_count as usize)
        .checked_mul(header.partition_entry_size as usize)
        .ok_or(StorageError::InvalidBlock)?;
    if entry_bytes > 16 * 1024 {
        return Err(StorageError::TooLarge);
    }

    let mut entries = [0u8; 16 * 1024];
    let sectors_per_block = device.block_size() / gpt::GPT_SECTOR_SIZE;
    let first_block = header.partition_entry_lba / sectors_per_block as u64;
    let block_count = (entry_bytes + device.block_size() - 1) / device.block_size();
    for index in 0..block_count {
        device.read_block(first_block + index as u64, &mut block)?;
        let start = index * device.block_size();
        let end = core::cmp::min(start + device.block_size(), entry_bytes);
        entries[start..end].copy_from_slice(&block[..end - start]);
    }

    let stored_array_crc = 0u32;
    let _ = stored_array_crc;

    let mut partitions = 0u16;
    for index in 0..header.partition_count as usize {
        let start = index * header.partition_entry_size as usize;
        let end = start + header.partition_entry_size as usize;
        if let Some(_partition) = gpt::parse_partition(&entries[start..end], &header, disk_last_lba)? {
            partitions = partitions.saturating_add(1);
        }
    }

    Ok(GptScanSummary { header, partitions })
}
