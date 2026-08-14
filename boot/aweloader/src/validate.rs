#![no_std]

use awe_boot_protocol::Architecture;
use crate::security::{validate_manifest, Manifest};

pub fn validate_aweos(manifest: &Manifest, image_len: u64, architecture: Architecture) -> bool {
    validate_manifest(manifest, image_len, architecture)
        && manifest.kernel_entry != 0
        && manifest.image_size != 0
        && manifest.image_offset < image_len
}

pub fn is_canonical_x86_64(address: u64) -> bool {
    let upper = address >> 48;
    upper == 0 || upper == 0xffff
}
