#![no_std]

use awe_boot_protocol::{Architecture, BootInfo, AWE_BOOT_MAGIC, AWE_BOOT_VERSION};

pub const AWEOS_IMAGE_MAGIC: [u8; 8] = *b"AWEOS001";
pub const AWEOS_MANIFEST_VERSION: u16 = 1;
pub const FLAG_SIGNED: u32 = 1 << 0;
pub const FLAG_RELEASE: u32 = 1 << 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Manifest {
    pub magic: [u8; 8],
    pub version: u16,
    pub header_size: u16,
    pub boot_protocol: u32,
    pub architecture: u32,
    pub flags: u32,
    pub image_offset: u64,
    pub image_size: u64,
    pub kernel_entry: u64,
    pub min_boot_version: u32,
    pub reserved: u32,
    pub image_digest: [u8; 32],
}

pub fn validate_manifest(m: &Manifest, image_len: u64, arch: Architecture) -> bool {
    if m.magic != AWEOS_IMAGE_MAGIC || m.version != AWEOS_MANIFEST_VERSION { return false; }
    if m.header_size as usize > core::mem::size_of::<Manifest>() { return false; }
    if m.boot_protocol != AWE_BOOT_VERSION { return false; }
    if m.min_boot_version > AWE_BOOT_VERSION { return false; }
    if m.flags & FLAG_SIGNED == 0 { return false; }
    if m.image_size == 0 { return false; }
    let end = match m.image_offset.checked_add(m.image_size) { Some(v) => v, None => return false };
    if end > image_len { return false; }
    if m.architecture != arch as u32 { return false; }
    true
}

pub fn validate_boot_info(info: &BootInfo, arch: Architecture) -> bool {
    info.magic == AWE_BOOT_MAGIC
        && info.version == AWE_BOOT_VERSION
        && info.architecture == arch
        && info.kernel_size != 0
        && info.kernel_base.checked_add(info.kernel_size).is_some()
}

/// Cryptographic verification is intentionally not faked here. A production
/// image must be verified by a reviewed Ed25519/secure-boot implementation
/// before this policy is considered complete.
pub fn signature_required(m: &Manifest) -> bool { m.flags & FLAG_SIGNED != 0 }
