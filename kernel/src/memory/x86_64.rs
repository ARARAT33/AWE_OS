#![no_std]

//! Minimal x86_64 paging activation for the first real CellKernel runtime.
//!
//! The bootstrap map is intentionally conservative: it identity-maps the
//! first 1 GiB with 2 MiB pages. This keeps the loader/kernel/Multiboot data
//! reachable while the permanent physical-frame allocator is brought online.
//! It is a bootstrap facility, not the final userspace address-space manager.

use core::arch::asm;

use super::paging::{PageFlags, PageTableEntry};

const ENTRIES: usize = 512;
const HUGE_2M: u64 = 1 << 7;
const PRESENT_WRITE: u64 = PageFlags::PRESENT.0 | PageFlags::WRITABLE.0 | HUGE_2M;

#[repr(C, align(4096))]
struct Table([u64; ENTRIES]);

impl Table {
    const fn empty() -> Self { Self([0; ENTRIES]) }
}

static mut PML4: Table = Table::empty();
static mut PDPT: Table = Table::empty();
static mut PD: Table = Table::empty();

/// Prepare and activate the bootstrap identity map.
///
/// # Safety
/// Must execute in x86_64 long mode with interrupts disabled. The first 1 GiB
/// of physical memory must remain identity-addressable for the lifetime of the
/// bootstrap phase.
#[inline(never)]
pub unsafe fn activate_bootstrap_identity_map() {
    let pml4 = core::ptr::addr_of_mut!(PML4);
    let pdpt = core::ptr::addr_of_mut!(PDPT);
    let pd = core::ptr::addr_of_mut!(PD);

    // PML4[0] -> PDPT, PDPT[0] -> PD.
    (*pml4).0[0] = (pdpt as u64) | PRESENT_WRITE;
    (*pdpt).0[0] = (pd as u64) | PRESENT_WRITE;

    // Identity-map 0..1GiB with 2MiB leaf entries.
    let mut index = 0usize;
    while index < ENTRIES {
        let physical = (index as u64) * 0x20_0000;
        (*pd).0[index] = physical | PRESENT_WRITE;
        index += 1;
    }

    let cr3 = pml4 as u64;
    asm!("mov cr3, {0}", in(reg) cr3, options(nostack, preserves_flags));
}

/// Return the physical address encoded in a page-table entry.
pub const fn bootstrap_entry_address(entry: u64) -> u64 {
    PageTableEntry::new(entry & 0x000f_ffff_ffff_f000, PageFlags::empty()).address()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_megabyte_leafs_cover_one_gib() {
        assert_eq!(ENTRIES as u64 * 0x20_0000, 0x4000_0000);
        assert_eq!(PRESENT_WRITE & PageFlags::PRESENT.0, PageFlags::PRESENT.0);
        assert_eq!(PRESENT_WRITE & HUGE_2M, HUGE_2M);
    }

    #[test]
    fn entry_address_masks_flags() {
        assert_eq!(bootstrap_entry_address(0x1234_5000 | PRESENT_WRITE), 0x1234_5000);
    }
}
