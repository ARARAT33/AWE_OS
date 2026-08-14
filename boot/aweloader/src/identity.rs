#![no_std]

pub const AWEOS_ID_MAGIC: [u8; 8] = *b"AWEOS001";
pub const AWEOS_BOOT_VERSION: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AweOsManifest {
    pub magic: [u8; 8],
    pub version: u32,
    pub architecture: u32,
    pub kernel_offset: u64,
    pub kernel_size: u64,
    pub manifest_size: u32,
    pub flags: u32,
}

pub fn is_aweos_manifest(m: &AweOsManifest) -> bool {
    m.magic == AWEOS_ID_MAGIC
        && m.version == AWEOS_BOOT_VERSION
        && m.kernel_size != 0
        && m.kernel_offset.checked_add(m.kernel_size).is_some()
}

/// The loader is intentionally AWEOS-only: it accepts an image only after
/// the AWEOS identity envelope has been validated. Foreign OS images are
/// rejected rather than chain-loaded.
pub fn accepts_only_aweos(m: &AweOsManifest) -> bool {
    is_aweos_manifest(m)
}
