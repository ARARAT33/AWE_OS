#![no_std]

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(pub u64);

impl VirtualAddress {
    pub const PAGE_SIZE: u64 = 4096;
    pub const fn is_aligned(self) -> bool { self.0 & (Self::PAGE_SIZE - 1) == 0 }
    pub const fn align_down(self) -> Self { Self(self.0 & !(Self::PAGE_SIZE - 1)) }
    pub const fn align_up(self) -> Option<Self> {
        let mask = Self::PAGE_SIZE - 1;
        match self.0.checked_add(mask) { Some(v) => Some(Self(v & !mask)), None => None }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(pub u64);

impl PhysicalAddress {
    pub const PAGE_SIZE: u64 = 4096;
    pub const fn is_aligned(self) -> bool { self.0 & (Self::PAGE_SIZE - 1) == 0 }
    pub const fn align_down(self) -> Self { Self(self.0 & !(Self::PAGE_SIZE - 1)) }
    pub const fn align_up(self) -> Option<Self> {
        let mask = Self::PAGE_SIZE - 1;
        match self.0.checked_add(mask) { Some(v) => Some(Self(v & !mask)), None => None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_is_overflow_safe() {
        assert_eq!(VirtualAddress(0x1234).align_down().0, 0x1000);
        assert_eq!(VirtualAddress(0x1001).align_up().unwrap().0, 0x2000);
        assert!(VirtualAddress(u64::MAX).align_up().is_none());
        assert!(PhysicalAddress(0x4000).is_aligned());
    }
}
