#![no_std]

use super::contract::DeviceContract;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(pub u64);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
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
    next_id: u64,
}

impl<const N: usize, const M: usize> DriverBus<N, M> {
    pub const fn new() -> Self { Self { entries: [None; N], len: 0, next_id: 1 } }

    pub fn register(&mut self, kind: DeviceKind, contract: DeviceContract<M>) -> Option<DeviceId> {
        if self.len == N || !contract.valid() { return None; }
        let id = DeviceId(self.next_id);
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
        assert!(bus.register(DeviceKind::Virtio, contract()).is_some());
        assert!(bus.register(DeviceKind::Virtio, contract()).is_none());
        assert_eq!(bus.len(), 1);
    }
}
