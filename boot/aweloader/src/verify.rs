#![no_std]

use core::convert::TryInto;

/// Minimal, dependency-free parser for the AWE kernel image envelope.
/// Production signing is deliberately separated from parsing so cryptography
/// can be replaced by a reviewed implementation without changing the loader ABI.
pub const IMAGE_MAGIC: [u8; 8] = *b"AWEKIMG1";
pub const IMAGE_VERSION: u16 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ImageHeader {
    pub magic: [u8; 8],
    pub version: u16,
    pub header_size: u16,
    pub architecture: u32,
    pub flags: u32,
    pub image_offset: u64,
    pub image_size: u64,
    pub entry: u64,
    pub load_address: u64,
}

pub fn parse_header(bytes: &[u8]) -> Option<ImageHeader> {
    if bytes.len() < core::mem::size_of::<ImageHeader>() {
        return None;
    }
    let h = unsafe { core::ptr::read_unaligned(bytes.as_ptr() as *const ImageHeader) };
    if h.magic != IMAGE_MAGIC || h.version != IMAGE_VERSION {
        return None;
    }
    if h.header_size as usize > bytes.len() || h.image_size == 0 {
        return None;
    }
    let end = h.image_offset.checked_add(h.image_size)?;
    if end > bytes.len() as u64 {
        return None;
    }
    let _ = h.entry.checked_add(0)?.try_into().ok();
    Some(h)
}

pub fn ranges_overlap(a_base: u64, a_len: u64, b_base: u64, b_len: u64) -> bool {
    let a_end = match a_base.checked_add(a_len) { Some(v) => v, None => return true };
    let b_end = match b_base.checked_add(b_len) { Some(v) => v, None => return true };
    a_base < b_end && b_base < a_end
}
