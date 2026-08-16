#![no_std]

use super::paging::{valid_mapping, PageFlags, PageTable, PageTableEntry};

/// A small, allocation-free mapper used by early kernel boot. It deliberately
/// owns only one level; higher levels are wired by the architecture bootstrap.
/// This keeps early paging deterministic and prevents accidental heap use.
pub struct PageMapper<'a> {
    table: &'a mut PageTable,
}

impl<'a> PageMapper<'a> {
    pub fn new(table: &'a mut PageTable) -> Self { Self { table } }

    pub fn map(&mut self, index: usize, physical: u64, flags: PageFlags) -> Result<(), MapError> {
        if !valid_mapping(physical, flags) { return Err(MapError::InvalidMapping); }
        if self.table.get(index).map(|e| e.is_present()).unwrap_or(true) {
            return Err(MapError::AlreadyMapped);
        }
        if !self.table.set(index, PageTableEntry::new(physical, flags.union(PageFlags::PRESENT))) {
            return Err(MapError::InvalidIndex);
        }
        Ok(())
    }

    pub fn unmap(&mut self, index: usize) -> Result<PageTableEntry, MapError> {
        let entry = self.table.get(index).ok_or(MapError::InvalidIndex)?;
        if !entry.is_present() { return Err(MapError::NotMapped); }
        self.table.clear(index);
        Ok(entry)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapError {
    InvalidMapping,
    AlreadyMapped,
    InvalidIndex,
    NotMapped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper_rejects_duplicate_and_unaligned_pages() {
        let mut table = PageTable::new();
        let mut mapper = PageMapper::new(&mut table);
        assert!(mapper.map(3, 0x3000, PageFlags::WRITABLE).is_ok());
        assert_eq!(mapper.map(3, 0x4000, PageFlags::WRITABLE), Err(MapError::AlreadyMapped));
        assert_eq!(mapper.map(4, 0x4001, PageFlags::WRITABLE), Err(MapError::InvalidMapping));
    }

    #[test]
    fn mapper_unmaps_and_returns_original_entry() {
        let mut table = PageTable::new();
        let mut mapper = PageMapper::new(&mut table);
        mapper.map(7, 0x7000, PageFlags::WRITABLE).unwrap();
        let old = mapper.unmap(7).unwrap();
        assert_eq!(old.address(), 0x7000);
        assert!(mapper.unmap(7).is_err());
    }
}
