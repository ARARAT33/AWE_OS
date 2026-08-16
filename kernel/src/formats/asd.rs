//! ASD (AWE System Driver) container format.
//! Dependency-free, bounded and `no_std` friendly.

pub const MAGIC: [u8; 4] = *b"ASD1";
pub const VERSION: u16 = 1;
pub const HEADER_LEN: usize = 32;
pub const MAX_METADATA: usize = 4096;
pub const FLAG_SIGNED: u16 = 1 << 0;
pub const FLAG_PRIVILEGED: u16 = 1 << 1;
pub const FLAG_USER_MODE: u16 = 1 << 2;
pub const KNOWN_FLAGS: u16 = FLAG_SIGNED | FLAG_PRIVILEGED | FLAG_USER_MODE;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error { TooSmall, BadMagic, BadVersion, InvalidHeader, OutOfBounds }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    pub version: u16, pub flags: u16, pub abi: u32, pub metadata_len: u32,
    pub payload_len: u32, pub signature_len: u32, pub entry_offset: u32, pub reserved: u32,
}

impl Header {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_LEN { return Err(Error::TooSmall); }
        if bytes[0..4] != MAGIC { return Err(Error::BadMagic); }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != VERSION { return Err(Error::BadVersion); }
        let h = Self {
            version,
            flags: u16::from_le_bytes([bytes[6], bytes[7]]),
            abi: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            metadata_len: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            payload_len: u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            signature_len: u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            entry_offset: u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            reserved: u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
        };
        if h.reserved != 0 || h.metadata_len as usize > MAX_METADATA || h.flags & !KNOWN_FLAGS != 0 {
            return Err(Error::InvalidHeader);
        }
        if h.payload_len == 0 || h.entry_offset >= h.payload_len { return Err(Error::InvalidHeader); }
        if h.flags & FLAG_SIGNED != 0 && h.signature_len == 0 { return Err(Error::InvalidHeader); }
        Ok(h)
    }

    pub fn total_len(&self) -> Result<usize, Error> {
        HEADER_LEN.checked_add(self.metadata_len as usize)
            .and_then(|v| v.checked_add(self.payload_len as usize))
            .and_then(|v| v.checked_add(self.signature_len as usize))
            .ok_or(Error::OutOfBounds)
    }
}

pub struct Image<'a> { pub header: Header, pub metadata: &'a [u8], pub payload: &'a [u8], pub signature: &'a [u8] }

impl<'a> Image<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        let header = Header::parse(bytes)?;
        let total = header.total_len()?;
        if bytes.len() < total { return Err(Error::OutOfBounds); }
        let m0 = HEADER_LEN;
        let m1 = m0 + header.metadata_len as usize;
        let p1 = m1 + header.payload_len as usize;
        let s1 = p1 + header.signature_len as usize;
        Ok(Self { header, metadata: &bytes[m0..m1], payload: &bytes[m1..p1], signature: &bytes[p1..s1] })
    }
}
