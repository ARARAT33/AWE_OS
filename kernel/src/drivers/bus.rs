#![no_std]

use super::contract::DeviceContract;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DeviceId {
    pub vendor: u16,
    pub device: u16,
    pub class: u32,
    pub revision: u8,
}

impl DeviceId {
    pub const fn packed(self) -> u64 {
        (self.vendor as u64)
            | ((self.device as u64) << 16)
            | ((self.class as u64) << 32)
            | ((self.revision as u64) << 56)
    }

    pub const fn valid(&self) -> bool {
        self.vendor != 0 && self.vendor != 0xffff && self.device != 0 && self.device != 0xffff
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeviceKind { Unknown = 0, Pci = 1, Virtio = 2, Platform = 3, Usb = 4 }

#[derive(Clone, Copy)]
pub struct BusEntry<const M: usize> {
    pub id: DeviceId,
    pub kind: DeviceKind,
    pub contract: DeviceContract<M>,
}

pub struct DriverBus<const N: usize, const M: usize> {
    entries: [Option<BusEntry<M>>; N],
    len: usize,
    next_id: u16,
}

impl<const N: usize, const M: usize> DriverBus<N, M> {
    pub const fn new() -> Self { Self { entries: [None; N], len: 0, next_id: 1 } }

    pub fn register(&mut self, kind: DeviceKind, contract: DeviceContract<M>) -> Option<DeviceId> {
        if self.len == N || !contract.valid() { return None; }
        let id = DeviceId {
            vendor: contract.vendor,
            device: self.next_id.max(1),
            class: contract.class_code,
            revision: kind as u8,
        };
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.entries[self.len] = Some(BusEntry { id, kind, contract });
        self.len += 1;
        Some(id)
    }

    pub const fn len(&self) -> usize { self.len }
    pub const fn is_full(&self) -> bool { self.len == N }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::contract::*;

    fn contract() -> DeviceContract<1> {
        DeviceContract {
            vendor: 0x1af4, device: 1, class_code: 0,
            mmio: [Some(MmioRegion { base: 0x1000, length: 0x100 })],
            interrupt: InterruptMode::MsiX,
            dma: DmaPolicy { max_bytes: 1 << 20, address_bits: 64, coherent: true },
        }
    }

    #[test]
    fn bus_bounds_devices() {
        let mut bus: DriverBus<1, 1> = DriverBus::new();
        let id = bus.register(DeviceKind::Virtio, contract()).unwrap();
        assert!(id.valid());
        assert!(bus.register(DeviceKind::Virtio, contract()).is_none());
        assert_eq!(bus.len(), 1);
    }
}
