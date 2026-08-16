#![no_std]

/// x86_64 page-table entry flags used by the kernel's initial paging layer.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageFlags(pub u64);

impl PageFlags {
    pub const PRESENT: Self = Self(1 << 0);
    pub const WRITABLE: Self = Self(1 << 1);
    pub const USER: Self = Self(1 << 2);
    pub const WRITE_THROUGH: Self = Self(1 << 3);
    pub const CACHE_DISABLE: Self = Self(1 << 4);
    pub const HUGE: Self = Self(1 << 7);
    pub const GLOBAL: Self = Self(1 << 8);
    pub const NO_EXECUTE: Self = Self(1 << 63);

    pub const fn empty() -> Self { Self(0) }
    pub const fn contains(self, other: Self) -> bool { (self.0 & other.0) == other.0 }
    pub const fn union(self, other: Self) -> Self { Self(self.0 | other.0) }
    pub const fn without(self, other: Self) -> Self { Self(self.0 & !other.0) }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;

    pub const fn empty() -> Self { Self(0) }

    pub const fn new(physical: u64, flags: PageFlags) -> Self {
        Self((physical & Self::ADDRESS_MASK) | flags.0)
    }

    pub const fn is_present(self) -> bool { self.0 & PageFlags::PRESENT.0 != 0 }
    pub const fn address(self) -> u64 { self.0 & Self::ADDRESS_MASK }
    pub const fn flags(self) -> PageFlags {
        PageFlags(self.0 & !Self::ADDRESS_MASK)
    }
}

/// A 4-level x86_64 page table. The table itself is deliberately a plain
/// `#[repr(C)]` value so the architecture layer can place it in page-aligned
/// memory and later activate it through CR3.
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        Self { entries: [PageTableEntry::empty(); 512] }
    }

    pub const fn len(&self) -> usize { self.entries.len() }

    pub fn get(&self, index: usize) -> Option<PageTableEntry> {
        if index < self.entries.len() { Some(self.entries[index]) } else { None }
    }

    pub fn set(&mut self, index: usize, entry: PageTableEntry) -> bool {
        if index >= self.entries.len() { return false; }
        self.entries[index] = entry;
        true
    }

    pub fn clear(&mut self, index: usize) -> bool {
        self.set(index, PageTableEntry::empty())
    }
}

/// Split a canonical 48-bit virtual address into the four 9-bit page-table
/// indices used by x86_64 long mode.
pub const fn indices(virtual_address: u64) -> [usize; 4] {
    [
        ((virtual_address >> 39) & 0x1ff) as usize,
        ((virtual_address >> 30) & 0x1ff) as usize,
        ((virtual_address >> 21) & 0x1ff) as usize,
        ((virtual_address >> 12) & 0x1ff) as usize,
    ]
}

pub const fn page_offset(virtual_address: u64) -> u16 {
    (virtual_address & 0xfff) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_and_flags_round_trip() {
        let flags = PageFlags::PRESENT.union(PageFlags::WRITABLE);
        let entry = PageTableEntry::new(0x1234_5000, flags);
        assert!(entry.is_present());
        assert_eq!(entry.address(), 0x1234_5000);
        assert!(entry.flags().contains(PageFlags::WRITABLE));
    }

    #[test]
    fn virtual_address_is_split_into_four_levels() {
        let value = 0x0000_7f12_3456_789a;
        let i = indices(value);
        assert_eq!(i[0], ((value >> 39) & 0x1ff) as usize);
        assert_eq!(i[1], ((value >> 30) & 0x1ff) as usize);
        assert_eq!(i[2], ((value >> 21) & 0x1ff) as usize);
        assert_eq!(i[3], ((value >> 12) & 0x1ff) as usize);
        assert_eq!(page_offset(value), 0x89a);
    }

    #[test]
    fn page_table_has_512_entries() {
        let table = PageTable::new();
        assert_eq!(table.len(), 512);
        assert!(!table.get(0).unwrap().is_present());
        assert!(table.get(512).is_none());
    }
}
