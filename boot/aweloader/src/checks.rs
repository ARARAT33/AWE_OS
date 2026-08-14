#![no_std]

use awe_boot_protocol::BootInfo;

pub fn boot_info_valid(info: &BootInfo) -> bool {
    awe_boot_protocol::validate(info)
        && info.kernel_base.checked_add(info.kernel_size).is_some()
        && info.memory_region_count < 1_000_000
}

pub fn address_range_valid(base: u64, size: u64) -> bool {
    size != 0 && base.checked_add(size).is_some()
}
