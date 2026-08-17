//! AWOS native application container.
//! Dependency-free, bounded and `no_std` friendly.

pub const MAGIC: [u8; 4] = *b"AWOS";
pub const VERSION: u16 = 1;
pub const HEADER_LEN: usize = 40;
pub const MAX_MANIFEST: usize = 8192;
pub const FLAG_SIGNED: u16 = 1 << 0;
pub const KNOWN_FLAGS: u16 = FLAG_SIGNED;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    TooSmall,
    BadMagic,
    BadVersion,
    InvalidHeader,
    OutOfBounds,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    pub version: u16,
    pub flags: u16,
    pub abi: u32,
    pub manifest_len: u32,
    pub code_len: u32,
    pub data_len: u32,
    pub signature_len: u32,
    pub entry_offset: u32,
    pub reserved: u32,
}

impl Header {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_LEN {
            return Err(Error::TooSmall);
        }
        if bytes[0..4] != MAGIC {
            return Err(Error::BadMagic);
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != VERSION {
            return Err(Error::BadVersion);
        }
        let h = Self {
            version,
            flags: u16::from_le_bytes([bytes[6], bytes[7]]),
            abi: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            manifest_len: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            code_len: u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            data_len: u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            signature_len: u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            entry_offset: u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
            reserved: u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]),
        };
        if h.reserved != 0
            || h.flags & !KNOWN_FLAGS != 0
            || h.manifest_len as usize > MAX_MANIFEST
            || h.code_len == 0
        {
            return Err(Error::InvalidHeader);
        }
        if h.entry_offset >= h.code_len {
            return Err(Error::InvalidHeader);
        }
        if h.flags & FLAG_SIGNED != 0 && h.signature_len == 0 {
            return Err(Error::InvalidHeader);
        }
        Ok(h)
    }

    pub fn total_len(&self) -> Result<usize, Error> {
        HEADER_LEN
            .checked_add(self.manifest_len as usize)
            .and_then(|v| v.checked_add(self.code_len as usize))
            .and_then(|v| v.checked_add(self.data_len as usize))
            .and_then(|v| v.checked_add(self.signature_len as usize))
            .ok_or(Error::OutOfBounds)
    }
}

pub struct Package<'a> {
    pub header: Header,
    pub manifest: &'a [u8],
    pub code: &'a [u8],
    pub data: &'a [u8],
    pub signature: &'a [u8],
}

impl<'a> Package<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        let header = Header::parse(bytes)?;
        let total = header.total_len()?;
        if bytes.len() < total {
            return Err(Error::OutOfBounds);
        }
        let m0 = HEADER_LEN;
        let m1 = m0 + header.manifest_len as usize;
        let c1 = m1 + header.code_len as usize;
        let d1 = c1 + header.data_len as usize;
        let s1 = d1 + header.signature_len as usize;
        Ok(Self {
            header,
            manifest: &bytes[m0..m1],
            code: &bytes[m1..c1],
            data: &bytes[c1..d1],
            signature: &bytes[d1..s1],
        })
    }
}
