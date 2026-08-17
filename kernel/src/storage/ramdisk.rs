//! Fixed-capacity in-memory block device used for storage integration tests.
//!
//! This is intentionally deterministic and allocation-free so filesystem and
//! partition logic can be exercised without depending on a host filesystem.

use super::{BlockDevice, StorageError, BLOCK_SIZE};

pub const RAMDISK_BLOCKS: usize = 64;

pub struct RamBlockDevice {
    data: [u8; RAMDISK_BLOCKS * BLOCK_SIZE],
    read_only: bool,
    dirty: bool,
}

impl RamBlockDevice {
    pub const fn new(read_only: bool) -> Self {
        Self {
            data: [0; RAMDISK_BLOCKS * BLOCK_SIZE],
            read_only,
            dirty: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn block_offset(block: u64) -> Result<usize, StorageError> {
        if block >= RAMDISK_BLOCKS as u64 {
            return Err(StorageError::InvalidBlock);
        }
        (block as usize)
            .checked_mul(BLOCK_SIZE)
            .ok_or(StorageError::InvalidBlock)
    }
}

impl Default for RamBlockDevice {
    fn default() -> Self {
        Self::new(false)
    }
}

impl BlockDevice for RamBlockDevice {
    fn block_count(&self) -> u64 {
        RAMDISK_BLOCKS as u64
    }

    fn read_block(&mut self, block: u64, out: &mut [u8]) -> Result<(), StorageError> {
        if out.len() < BLOCK_SIZE {
            return Err(StorageError::BufferTooSmall);
        }
        let offset = Self::block_offset(block)?;
        out[..BLOCK_SIZE].copy_from_slice(&self.data[offset..offset + BLOCK_SIZE]);
        Ok(())
    }

    fn write_block(&mut self, block: u64, data: &[u8]) -> Result<(), StorageError> {
        if self.read_only {
            return Err(StorageError::ReadOnly);
        }
        if data.len() < BLOCK_SIZE {
            return Err(StorageError::BufferTooSmall);
        }
        let offset = Self::block_offset(block)?;
        self.data[offset..offset + BLOCK_SIZE].copy_from_slice(&data[..BLOCK_SIZE]);
        self.dirty = true;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), StorageError> {
        self.dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_flush_are_deterministic() {
        let mut disk = RamBlockDevice::default();
        let mut input = [0u8; BLOCK_SIZE];
        input[0] = 0xA5;
        input[BLOCK_SIZE - 1] = 0x5A;
        disk.write_block(3, &input).expect("write");
        assert!(disk.is_dirty());

        let mut output = [0u8; BLOCK_SIZE];
        disk.read_block(3, &mut output).expect("read");
        assert_eq!(output, input);
        disk.flush().expect("flush");
        assert!(!disk.is_dirty());
    }

    #[test]
    fn enforces_bounds_and_read_only_mode() {
        let mut disk = RamBlockDevice::new(true);
        let block = [0u8; BLOCK_SIZE];
        assert_eq!(disk.write_block(0, &block), Err(StorageError::ReadOnly));
        assert_eq!(disk.read_block(RAMDISK_BLOCKS as u64, &mut [0; BLOCK_SIZE]), Err(StorageError::InvalidBlock));
        assert_eq!(disk.read_block(0, &mut [0; BLOCK_SIZE - 1]), Err(StorageError::BufferTooSmall));
    }
}
