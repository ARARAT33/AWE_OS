#![no_std]

pub const AWEOS_MAGIC: [u8; 8] = *b"AWEOS001";
pub const MIN_BOOT_PROTOCOL: u32 = 1;
pub const MAX_BOOT_PROTOCOL: u32 = 1;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86 = 1,
    X86_64 = 2,
    Arm = 3,
    Aarch64 = 4,
    RiscV32 = 5,
    RiscV64 = 6,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Manifest {
    pub magic: [u8; 8],
    pub version: u32,
    pub protocol: u32,
    pub architecture: u32,
    pub image_size: u64,
    pub kernel_offset: u64,
    pub kernel_size: u64,
    pub entry: u64,
    pub image_version: u64,
    pub flags: u64,
}

pub fn valid_basic(m: &Manifest, image_len: u64) -> bool {
    m.magic == AWEOS_MAGIC
        && m.protocol >= MIN_BOOT_PROTOCOL
        && m.protocol <= MAX_BOOT_PROTOCOL
        && m.image_size <= image_len
        && m.kernel_size != 0
        && m.kernel_offset.checked_add(m.kernel_size).map_or(false, |e| e <= m.image_size)
        && m.entry != 0
}

pub fn architecture_matches(manifest_arch: u32, running_arch: Architecture) -> bool {
    manifest_arch == running_arch as u32
}
