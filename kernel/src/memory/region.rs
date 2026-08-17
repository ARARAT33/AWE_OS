#![no_std]

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    Usable = 1,
    Reserved = 2,
    Reclaimable = 3,
    Mmio = 4,
}

#[derive(Clone, Copy)]
pub struct Region {
    pub base: u64,
    pub length: u64,
    pub kind: RegionKind,
}

impl Region {
    pub const fn end(&self) -> Option<u64> {
        self.base.checked_add(self.length)
    }

    pub const fn contains(&self, address: u64) -> bool {
        match self.end() {
            Some(end) => address >= self.base && address < end,
            None => false,
        }
    }
}
