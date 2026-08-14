#![no_std]

use awe_boot_protocol::MemoryRegion;

pub const USABLE: u32 = 1;
pub const RESERVED: u32 = 2;
pub const RECLAIMABLE: u32 = 3;
pub const MMIO: u32 = 4;

pub fn normalize(base: u64, length: u64, kind: u32) -> Option<MemoryRegion> {
    base.checked_add(length)?;
    Some(MemoryRegion { base, length, kind, reserved: 0 })
}

pub fn overlaps(a: &MemoryRegion, b: &MemoryRegion) -> bool {
    let ae = match a.base.checked_add(a.length) { Some(v) => v, None => return true };
    let be = match b.base.checked_add(b.length) { Some(v) => v, None => return true };
    a.base < be && b.base < ae
}
