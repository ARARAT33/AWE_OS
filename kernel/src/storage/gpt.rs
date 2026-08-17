//! Bounded GPT header/partition parsing for the storage service boundary.
//!
//! The parser is deliberately allocation-free and validates both structural
//! bounds and CRC32 values before exposing partition metadata.

#![allow(dead_code)]

pub const GPT_SECTOR_SIZE: usize = 512;
pub const GPT_SIGNATURE: [u8; 8] = *b"EFI PART";
pub const GPT_REVISION_1_0: u32 = 0x0001_0000;
pub const GPT_HEADER_MIN_SIZE: usize = 92;
pub const GPT_MAX_PARTITIONS: usize = 128;
pub const GPT_PARTITION_ENTRY_MIN_SIZE: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GptError {
    BufferTooSmall,
    BadSignature,
    UnsupportedRevision,
    InvalidHeaderSize,
    InvalidLbaRange,
    InvalidEntrySize,
    TooManyPartitions,
    InvalidPartitionRange,
    HeaderCrcMismatch,
    PartitionArrayCrcMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GptHeader {
    pub revision: u32,
    pub header_size: u32,
    pub current_lba: u64,
    pub backup_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub partition_entry_lba: u64,
    pub partition_count: u32,
    pub partition_entry_size: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GptPartition {
    pub type_guid: [u8; 16],
    pub unique_guid: [u8; 16],
    pub first_lba: u64,
    pub last_lba: u64,
    pub attributes: u64,
}

pub fn parse_header(sector: &[u8], disk_last_lba: u64) -> Result<GptHeader, GptError> {
    if sector.len() < GPT_SECTOR_SIZE {
        return Err(GptError::BufferTooSmall);
    }
    if sector[0..8] != GPT_SIGNATURE {
        return Err(GptError::BadSignature);
    }

    let revision = le_u32(sector, 8);
    if revision < GPT_REVISION_1_0 {
        return Err(GptError::UnsupportedRevision);
    }
    let header_size = le_u32(sector, 12) as usize;
    if !(GPT_HEADER_MIN_SIZE..=GPT_SECTOR_SIZE).contains(&header_size) {
        return Err(GptError::InvalidHeaderSize);
    }

    let current_lba = le_u64(sector, 24);
    let backup_lba = le_u64(sector, 32);
    let first_usable_lba = le_u64(sector, 40);
    let last_usable_lba = le_u64(sector, 48);
    let partition_entry_lba = le_u64(sector, 72);
    let partition_count = le_u32(sector, 80);
    let partition_entry_size = le_u32(sector, 84) as usize;

    if current_lba > disk_last_lba
        || backup_lba > disk_last_lba
        || current_lba == backup_lba
        || first_usable_lba > last_usable_lba
        || last_usable_lba > disk_last_lba
        || partition_entry_lba > disk_last_lba
    {
        return Err(GptError::InvalidLbaRange);
    }
    if partition_count == 0
        || partition_count as usize > GPT_MAX_PARTITIONS
        || partition_entry_size < GPT_PARTITION_ENTRY_MIN_SIZE
        || !partition_entry_size.is_multiple_of(8)
    {
        return Err(if partition_count as usize > GPT_MAX_PARTITIONS {
            GptError::TooManyPartitions
        } else {
            GptError::InvalidEntrySize
        });
    }

    let stored_crc = le_u32(sector, 16);
    let mut header_copy = [0u8; GPT_SECTOR_SIZE];
    header_copy.copy_from_slice(&sector[..GPT_SECTOR_SIZE]);
    header_copy[16..20].fill(0);
    if crc32(&header_copy[..header_size]) != stored_crc {
        return Err(GptError::HeaderCrcMismatch);
    }

    Ok(GptHeader {
        revision,
        header_size: header_size as u32,
        current_lba,
        backup_lba,
        first_usable_lba,
        last_usable_lba,
        partition_entry_lba,
        partition_count,
        partition_entry_size: partition_entry_size as u32,
    })
}

pub fn parse_partition(
    entry: &[u8],
    header: &GptHeader,
    disk_last_lba: u64,
) -> Result<Option<GptPartition>, GptError> {
    if entry.len() < header.partition_entry_size as usize {
        return Err(GptError::BufferTooSmall);
    }
    let mut type_guid = [0u8; 16];
    let mut unique_guid = [0u8; 16];
    type_guid.copy_from_slice(&entry[..16]);
    if type_guid.iter().all(|&byte| byte == 0) {
        return Ok(None);
    }
    unique_guid.copy_from_slice(&entry[16..32]);
    let first_lba = le_u64(entry, 32);
    let last_lba = le_u64(entry, 40);
    if first_lba > last_lba
        || first_lba < header.first_usable_lba
        || last_lba > header.last_usable_lba
        || last_lba > disk_last_lba
    {
        return Err(GptError::InvalidPartitionRange);
    }

    Ok(Some(GptPartition {
        type_guid,
        unique_guid,
        first_lba,
        last_lba,
        attributes: le_u64(entry, 48),
    }))
}

pub fn validate_partition_array_crc(array: &[u8], expected: u32) -> Result<(), GptError> {
    if crc32(array) == expected {
        Ok(())
    } else {
        Err(GptError::PartitionArrayCrcMismatch)
    }
}

fn le_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn le_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_sector() -> [u8; GPT_SECTOR_SIZE] {
        let mut sector = [0u8; GPT_SECTOR_SIZE];
        sector[..8].copy_from_slice(&GPT_SIGNATURE);
        sector[8..12].copy_from_slice(&GPT_REVISION_1_0.to_le_bytes());
        sector[12..16].copy_from_slice(&(GPT_HEADER_MIN_SIZE as u32).to_le_bytes());
        sector[24..32].copy_from_slice(&1u64.to_le_bytes());
        sector[32..40].copy_from_slice(&999u64.to_le_bytes());
        sector[40..48].copy_from_slice(&34u64.to_le_bytes());
        sector[48..56].copy_from_slice(&900u64.to_le_bytes());
        sector[72..80].copy_from_slice(&2u64.to_le_bytes());
        sector[80..84].copy_from_slice(&1u32.to_le_bytes());
        sector[84..88].copy_from_slice(&(GPT_PARTITION_ENTRY_MIN_SIZE as u32).to_le_bytes());
        let crc = {
            let mut copy = sector;
            copy[16..20].fill(0);
            crc32(&copy[..GPT_HEADER_MIN_SIZE])
        };
        sector[16..20].copy_from_slice(&crc.to_le_bytes());
        sector
    }

    #[test]
    fn validates_header_and_partition() {
        let sector = header_sector();
        let header = parse_header(&sector, 999).expect("valid GPT header");
        let mut entry = [0u8; GPT_PARTITION_ENTRY_MIN_SIZE];
        entry[0] = 1;
        entry[16] = 2;
        entry[32..40].copy_from_slice(&34u64.to_le_bytes());
        entry[40..48].copy_from_slice(&100u64.to_le_bytes());
        let partition = parse_partition(&entry, &header, 999).expect("valid entry");
        assert!(partition.is_some());
    }

    #[test]
    fn rejects_tampered_header() {
        let mut sector = header_sector();
        sector[40] ^= 1;
        assert_eq!(parse_header(&sector, 999), Err(GptError::HeaderCrcMismatch));
    }

    #[test]
    fn rejects_out_of_range_partition() {
        let sector = header_sector();
        let header = parse_header(&sector, 999).expect("valid GPT header");
        let mut entry = [0u8; GPT_PARTITION_ENTRY_MIN_SIZE];
        entry[0] = 1;
        entry[32..40].copy_from_slice(&33u64.to_le_bytes());
        entry[40..48].copy_from_slice(&901u64.to_le_bytes());
        assert_eq!(
            parse_partition(&entry, &header, 999),
            Err(GptError::InvalidPartitionRange)
        );
    }
}
