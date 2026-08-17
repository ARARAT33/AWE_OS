#![no_std]

//! ACPI root-pointer and table-directory validation for driverd.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsdpError {
    TooShort,
    BadSignature,
    BadChecksum,
    BadLength,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RsdpInfo {
    pub revision: u8,
    pub rsdt_address: u32,
    pub xsdt_address: u64,
    pub length: u32,
}

pub fn validate_rsdp(bytes: &[u8]) -> Result<RsdpInfo, RsdpError> {
    if bytes.len() < 20 { return Err(RsdpError::TooShort); }
    if &bytes[0..8] != b"RSD PTR " { return Err(RsdpError::BadSignature); }
    let mut sum = 0u8;
    let mut i = 0usize;
    while i < 20 {
        sum = sum.wrapping_add(bytes[i]);
        i += 1;
    }
    if sum != 0 { return Err(RsdpError::BadChecksum); }
    let revision = bytes[15];
    let rsdt_address = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    if revision < 2 {
        return Ok(RsdpInfo { revision, rsdt_address, xsdt_address: 0, length: 20 });
    }
    if bytes.len() < 36 { return Err(RsdpError::TooShort); }
    let length = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if length < 36 || length as usize > bytes.len() { return Err(RsdpError::BadLength); }
    let mut extended_sum = 0u8;
    let mut j = 0usize;
    while j < length as usize {
        extended_sum = extended_sum.wrapping_add(bytes[j]);
        j += 1;
    }
    if extended_sum != 0 { return Err(RsdpError::BadChecksum); }
    let xsdt_address = u64::from_le_bytes([
        bytes[24], bytes[25], bytes[26], bytes[27],
        bytes[28], bytes[29], bytes[30], bytes[31],
    ]);
    Ok(RsdpInfo { revision, rsdt_address, xsdt_address, length })
}

pub fn parse_pointer_table<const N: usize>(
    bytes: &[u8],
    entry_size: usize,
    out: &mut [u64; N],
) -> Result<usize, RsdpError> {
    if entry_size != 4 && entry_size != 8 { return Err(RsdpError::BadLength); }
    if bytes.len() % entry_size != 0 { return Err(RsdpError::BadLength); }
    let count = core::cmp::min(bytes.len() / entry_size, N);
    let mut i = 0usize;
    while i < count {
        let base = i * entry_size;
        out[i] = if entry_size == 4 {
            u32::from_le_bytes([bytes[base], bytes[base + 1], bytes[base + 2], bytes[base + 3]]) as u64
        } else {
            u64::from_le_bytes([
                bytes[base], bytes[base + 1], bytes[base + 2], bytes[base + 3],
                bytes[base + 4], bytes[base + 5], bytes[base + 6], bytes[base + 7],
            ])
        };
        i += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_rsdp_signature() {
        let bytes = [0u8; 20];
        assert_eq!(validate_rsdp(&bytes), Err(RsdpError::BadSignature));
    }

    #[test]
    fn parses_pointer_table() {
        let bytes = [1u8, 0, 0, 0, 2, 0, 0, 0];
        let mut out = [0u64; 4];
        assert_eq!(parse_pointer_table(&bytes, 4, &mut out).unwrap(), 2);
        assert_eq!(out[1], 2);
    }
}
