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
