#![no_std]

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DriverKind {
    Bus = 0,
    Storage = 1,
    Network = 2,
    Input = 3,
    Display = 4,
    Audio = 5,
    Security = 6,
    Sensor = 7,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct DriverId {
    pub vendor: u16,
    pub device: u16,
    pub kind: DriverKind,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DriverDescriptor {
    pub id: DriverId,
    pub mmio_base: u64,
    pub mmio_len: u64,
    pub dma_mask: u64,
    pub irq: u32,
}

impl DriverDescriptor {
    pub const fn valid(&self) -> bool {
        self.mmio_len != 0
            && self.mmio_base.checked_add(self.mmio_len).is_some()
            && self.dma_mask != 0
    }
}

pub struct DriverRegistry<const N: usize> {
    entries: [Option<DriverDescriptor>; N],
    len: usize,
}

impl<const N: usize> DriverRegistry<N> {
    pub const fn new() -> Self { Self { entries: [None; N], len: 0 } }

    pub fn register(&mut self, descriptor: DriverDescriptor) -> bool {
        if !descriptor.valid() || self.len == N { return false; }
        if self.entries[..self.len].iter().any(|entry| entry.map(|e| e.id) == Some(descriptor.id)) { return false; }
        self.entries[self.len] = Some(descriptor);
        self.len += 1;
        true
    }

    pub const fn len(&self) -> usize { self.len }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: DriverId = DriverId { vendor: 0x1AF4, device: 0x1001, kind: DriverKind::Storage };
    const DESC: DriverDescriptor = DriverDescriptor { id: ID, mmio_base: 0x1000, mmio_len: 0x1000, dma_mask: 0xFFFF_FFFF, irq: 5 };

    #[test]
    fn registry_rejects_duplicates_and_bad_ranges() {
        let mut registry: DriverRegistry<2> = DriverRegistry::new();
        assert!(registry.register(DESC));
        assert!(!registry.register(DESC));
        let bad = DriverDescriptor { id: DriverId { vendor: 1, device: 2, kind: DriverKind::Bus }, mmio_base: u64::MAX, mmio_len: 2, dma_mask: 1, irq: 0 };
        assert!(!registry.register(bad));
    }
}
