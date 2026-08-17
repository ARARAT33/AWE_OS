#![no_std]

//! AWE 64.0 hardware-boundary access contracts.
//!
//! These primitives define how a trusted driver may describe MMIO/PIO access,
//! interrupt ownership and power-management intent. They do not perform PCI,
//! ACPI, APIC, VirtIO, DMA or hardware discovery; those remain reserved for
//! the 65% execution checkpoint.

use super::DeviceId;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessKind {
    Mmio = 0,
    Pio = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IoRegion {
    pub device: DeviceId,
    pub base: u64,
    pub length: u64,
    pub kind: AccessKind,
}

impl IoRegion {
    pub const fn new(device: DeviceId, base: u64, length: u64, kind: AccessKind) -> Self {
        Self { device, base, length, kind }
    }

    pub const fn valid(self) -> bool {
        self.length != 0 && self.base.checked_add(self.length).is_some()
    }

    pub const fn contains(self, offset: u64, width: u64) -> bool {
        if width == 0 || offset > self.length { return false; }
        offset.checked_add(width).map_or(false, |end| end <= self.length)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrqMode {
    Line = 0,
    Msi = 1,
    MsiX = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterruptGrant {
    pub device: DeviceId,
    pub vector: u32,
    pub mode: IrqMode,
    pub shared: bool,
}

impl InterruptGrant {
    pub const fn valid(self) -> bool { self.vector != 0 }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerState {
    D0 = 0,
    D1 = 1,
    D2 = 2,
    D3 = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerTransitionError {
    Invalid,
    Unsupported,
}

pub const fn power_transition(current: PowerState, next: PowerState) -> Result<(), PowerTransitionError> {
    if current as u8 == next as u8 { return Ok(()); }
    if matches!((current, next),
        (PowerState::D0, PowerState::D1)
        | (PowerState::D1, PowerState::D0)
        | (PowerState::D1, PowerState::D2)
        | (PowerState::D2, PowerState::D1)
        | (PowerState::D2, PowerState::D3)
        | (PowerState::D3, PowerState::D2)) {
        Ok(())
    } else {
        Err(PowerTransitionError::Invalid)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceAccessContract {
    pub device: DeviceId,
    pub mmio: IoRegion,
    pub pio: IoRegion,
    pub interrupt: InterruptGrant,
    pub power: PowerState,
}

impl DeviceAccessContract {
    pub const fn new(device: DeviceId, mmio: IoRegion, pio: IoRegion, interrupt: InterruptGrant) -> Self {
        Self { device, mmio, pio, interrupt, power: PowerState::D0 }
    }

    pub const fn valid(self) -> bool {
        self.mmio.device == self.device
            && self.pio.device == self.device
            && self.interrupt.device == self.device
            && self.mmio.valid()
            && self.pio.valid()
            && self.interrupt.valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_region_is_overflow_safe() {
        let r = IoRegion::new(DeviceId(1), 0x1000, 0x100, AccessKind::Mmio);
        assert!(r.valid());
        assert!(r.contains(0x20, 4));
        assert!(!r.contains(0xff, 2));
        let overflow = IoRegion::new(DeviceId(1), u64::MAX, 2, AccessKind::Mmio);
        assert!(!overflow.valid());
    }

    #[test]
    fn interrupt_grant_requires_vector() {
        let invalid = InterruptGrant { device: DeviceId(1), vector: 0, mode: IrqMode::Msi, shared: false };
        assert!(!invalid.valid());
        let valid = InterruptGrant { device: DeviceId(1), vector: 32, mode: IrqMode::MsiX, shared: false };
        assert!(valid.valid());
    }

    #[test]
    fn power_transitions_are_explicit() {
        assert_eq!(power_transition(PowerState::D0, PowerState::D1), Ok(()));
        assert_eq!(power_transition(PowerState::D0, PowerState::D3), Err(PowerTransitionError::Invalid));
        assert_eq!(power_transition(PowerState::D3, PowerState::D2), Ok(()));
    }

    #[test]
    fn access_contract_requires_consistent_device_identity() {
        let contract = DeviceAccessContract::new(
            DeviceId(4),
            IoRegion::new(DeviceId(4), 0x1000, 0x100, AccessKind::Mmio),
            IoRegion::new(DeviceId(4), 0x40, 0x20, AccessKind::Pio),
            InterruptGrant { device: DeviceId(4), vector: 33, mode: IrqMode::Msi, shared: false },
        );
        assert!(contract.valid());
        let bad = DeviceAccessContract::new(
            DeviceId(4),
            IoRegion::new(DeviceId(9), 0x1000, 0x100, AccessKind::Mmio),
            IoRegion::new(DeviceId(4), 0x40, 0x20, AccessKind::Pio),
            InterruptGrant { device: DeviceId(4), vector: 33, mode: IrqMode::Msi, shared: false },
        );
        assert!(!bad.valid());
    }
}
