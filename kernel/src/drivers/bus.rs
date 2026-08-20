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
    pub const fn new(vendor: u16, device: u16) -> Self {
        Self {
            vendor,
            device,
            class: 0,
            revision: 0,
        }
    }
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
pub enum DeviceKind {
    Unknown = 0,
    Pci = 1,
    Virtio = 2,
    Platform = 3,
    Usb = 4,
}
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
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            len: 0,
            next_id: 1,
        }
    }
    pub fn register(&mut self, kind: DeviceKind, contract: DeviceContract<M>) -> Option<DeviceId> {
        if self.len == N || !contract.valid() {
            return None;
        }
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
    pub const fn len(&self) -> usize {
        self.len
    }
    pub const fn is_full(&self) -> bool {
        self.len == N
    }
}
