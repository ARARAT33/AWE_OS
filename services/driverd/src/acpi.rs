#![no_std]

//! Minimal, strict ACPI table parser used by driverd for platform discovery.
//! Parsing is bounds-checked, checksum-verified and allocation-free.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcpiTableRef {
    pub signature: [u8; 4],
    pub address: u64,
    pub length: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcpiError {
    TooShort,
    BadChecksum,
    InvalidLength,
    TableOverflow,
}

pub fn checksum_ok(bytes: &[u8]) -> bool {
    let mut sum = 0u8;
    let mut i = 0usize;
    while i < bytes.len() {
        sum = sum.wrapping_add(bytes[i]);
        i += 1;
    }
    sum == 0
}

pub fn parse_header(bytes: &[u8]) -> Result<SdtHeader, AcpiError> {
    if bytes.len() < 10 { return Err(AcpiError::TooShort); }
    let length = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if length < 10 { return Err(AcpiError::InvalidLength); }
    if length as usize > bytes.len() { return Err(AcpiError::TableOverflow); }
    if !checksum_ok(&bytes[..length as usize]) { return Err(AcpiError::BadChecksum); }
    Ok(SdtHeader {
        signature: [bytes[0], bytes[1], bytes[2], bytes[3]],
        length,
        revision: bytes[8],
        checksum: bytes[9],
    })
}

pub fn find_table<const N: usize>(
    entries: &[AcpiTableRef],
    signature: [u8; 4],
    out: &mut [Option<AcpiTableRef>; N],
) -> usize {
    let mut count = 0usize;
    let mut i = 0usize;
    while i < entries.len() && count < N {
        if entries[i].signature == signature {
            out[count] = Some(entries[i]);
            count += 1;
        }
        i += 1;
    }
    count
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MadtRecord {
    pub local_apic_address: u32,
    pub flags: u32,
    pub record_count: u16,
}

pub fn parse_madt(body: &[u8]) -> Result<MadtRecord, AcpiError> {
    if body.len() < 8 { return Err(AcpiError::TooShort); }
    Ok(MadtRecord {
        local_apic_address: u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
        flags: u32::from_le_bytes([body[4], body[5], body[6], body[7]]),
        record_count: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_checksum_is_detected() {
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(b"TEST");
        bytes[4..8].copy_from_slice(&(12u32).to_le_bytes());
        assert!(checksum_ok(&bytes));
        bytes[11] = 1;
        assert!(!checksum_ok(&bytes));
    }

    #[test]
    fn table_search_is_bounded() {
        let entries = [
            AcpiTableRef { signature: *b"FACP", address: 1, length: 100 },
            AcpiTableRef { signature: *b"APIC", address: 2, length: 80 },
            AcpiTableRef { signature: *b"APIC", address: 3, length: 96 },
        ];
        let mut out = [None; 1];
        assert_eq!(find_table(&entries, *b"APIC", &mut out), 1);
        assert_eq!(out[0].unwrap().address, 2);
    }

    #[test]
    fn madt_header_is_parsed() {
        let body = [0x00, 0x00, 0xFE, 0x80, 0x01, 0, 0, 0];
        let madt = parse_madt(&body).unwrap();
        assert_eq!(madt.local_apic_address, 0x80FE0000);
        assert_eq!(madt.flags, 1);
    }
}
