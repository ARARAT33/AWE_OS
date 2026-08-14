//! Native AWEOS application format contracts.
//! The on-disk container will be versioned and signed; the runtime must validate
//! permissions before mapping executable pages.

pub const MAGIC: [u8; 8] = *b"AWESA\0\1\0";
pub const FORMAT_VERSION: u16 = 1;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum CapabilityKind {
    FilesystemRead = 1,
    FilesystemWrite = 2,
    Network = 3,
    Device = 4,
    Gpu = 5,
    Audio = 6,
    ProcessSpawn = 7,
    Ipc = 8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Header {
    pub magic: [u8; 8],
    pub version: u16,
    pub flags: u16,
    pub manifest_offset: u64,
    pub manifest_size: u64,
    pub code_offset: u64,
    pub code_size: u64,
    pub signature_offset: u64,
    pub signature_size: u64,
}

impl Header {
    pub const fn valid_magic(&self) -> bool { self.magic == MAGIC }
}
