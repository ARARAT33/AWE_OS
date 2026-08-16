#![no_std]

use super::mapper::{MapError, PageMapper};
use super::paging::{PageFlags, PageTable};

/// Early identity mapper used by the boot path. It maps a bounded physical
/// window page-by-page and refuses overflow, unaligned addresses, duplicate
/// mappings and requests that exceed the table capacity.
pub struct BootMapper<'a> {
    mapper: PageMapper<'a>,
}

impl<'a> BootMapper<'a> {
    pub fn new(table: &'a mut PageTable) -> Self { Self { mapper: PageMapper::new(table) } }

    pub fn identity_map_range(&mut self, start: u64, length: u64, flags: PageFlags) -> Result<usize, BootMapError> {
        if length == 0 || (start & 0xfff) != 0 || (length & 0xfff) != 0 { return Err(BootMapError::InvalidRange); }
        let end = start.checked_add(length).ok_or(BootMapError::Overflow)?;
        let pages = (end - start) / 4096;
        if pages > 512 { return Err(BootMapError::TableCapacity); }
        let mut i = 0usize;
        while i < pages as usize {
            let physical = start.checked_add((i as u64) * 4096).ok_or(BootMapError::Overflow)?;
            self.mapper.map(i, physical, flags).map_err(BootMapError::Map)?;
            i += 1;
        }
        Ok(i)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BootMapError { InvalidRange, Overflow, TableCapacity, Map(MapError) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_a_small_identity_window() {
        let mut table = PageTable::new();
        let mut mapper = BootMapper::new(&mut table);
        assert_eq!(mapper.identity_map_range(0x1000, 0x3000, PageFlags::WRITABLE).unwrap(), 3);
        assert_eq!(table.get(0).unwrap().address(), 0x1000);
        assert_eq!(table.get(2).unwrap().address(), 0x3000);
    }
    #[test]
    fn rejects_overflow_and_unaligned_ranges() {
        let mut table = PageTable::new();
        let mut mapper = BootMapper::new(&mut table);
        assert_eq!(mapper.identity_map_range(0x1001, 0x1000, PageFlags::empty()), Err(BootMapError::InvalidRange));
        assert_eq!(mapper.identity_map_range(u64::MAX - 0xfff, 0x2000, PageFlags::empty()), Err(BootMapError::Overflow));
    }
    #[test]
    fn rejects_more_than_one_table() {
        let mut table = PageTable::new();
        let mut mapper = BootMapper::new(&mut table);
        assert_eq!(mapper.identity_map_range(0, 513 * 4096, PageFlags::PRESENT), Err(BootMapError::TableCapacity));
    }
}
