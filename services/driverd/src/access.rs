//! Driver-service hardware access contract.
//!
//! This is the execution-facing metadata layer for 64.0. It does not touch
//! hardware; concrete PCI/ACPI/VirtIO discovery and programming remain in the
//! 65% checkpoint.

use crate::DriverId;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessKind {
    Mmio = 0,
    Pio = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessRegion {
    pub driver: DriverId,
    pub base: u64,
    pub length: u64,
    pub kind: AccessKind,
}

impl AccessRegion {
    pub fn valid(self) -> bool {
        self.length != 0 && self.base.checked_add(self.length).is_some()
    }

    pub fn contains(self, offset: u64, width: u64) -> bool {
        if width == 0 || offset > self.length {
            return false;
        }
        match offset.checked_add(width) {
            Some(end) => end <= self.length,
            None => false,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptMode {
    Line = 0,
    Msi = 1,
    MsiX = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterruptOwnership {
    pub driver: DriverId,
    pub vector: u32,
    pub mode: InterruptMode,
    pub shared: bool,
}

impl InterruptOwnership {
    pub fn valid(self) -> bool {
        self.vector != 0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerState {
    D0 = 0,
    D1 = 1,
    D2 = 2,
    D3 = 3,
}

pub const fn power_transition_allowed(current: PowerState, next: PowerState) -> bool {
    if current as u8 == next as u8 {
        return true;
    }
    matches!(
        (current, next),
        (PowerState::D0, PowerState::D1)
            | (PowerState::D1, PowerState::D0)
            | (PowerState::D1, PowerState::D2)
            | (PowerState::D2, PowerState::D1)
            | (PowerState::D2, PowerState::D3)
            | (PowerState::D3, PowerState::D2)
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareAccessPlan {
    pub driver: DriverId,
    pub mmio: AccessRegion,
    pub pio: AccessRegion,
    pub interrupt: InterruptOwnership,
    pub power: PowerState,
}

impl HardwareAccessPlan {
    pub fn valid(self) -> bool {
        self.mmio.driver == self.driver
            && self.pio.driver == self.driver
            && self.interrupt.driver == self.driver
            && self.mmio.valid()
            && self.pio.valid()
            && self.interrupt.valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_bounds_are_overflow_safe() {
        let region = AccessRegion {
            driver: DriverId(1),
            base: 0x1000,
            length: 0x100,
            kind: AccessKind::Mmio,
        };
        assert!(region.valid());
        assert!(region.contains(0x20, 4));
        assert!(!region.contains(0xff, 2));
        assert!(
            !AccessRegion {
                driver: DriverId(1),
                base: u64::MAX,
                length: 2,
                kind: AccessKind::Mmio,
            }
            .valid()
        );
    }

    #[test]
    fn interrupt_ownership_is_explicit() {
        assert!(
            !InterruptOwnership {
                driver: DriverId(1),
                vector: 0,
                mode: InterruptMode::Msi,
                shared: false,
            }
            .valid()
        );
        assert!(
            InterruptOwnership {
                driver: DriverId(1),
                vector: 32,
                mode: InterruptMode::MsiX,
                shared: false,
            }
            .valid()
        );
    }

    #[test]
    fn power_policy_is_bounded() {
        assert!(power_transition_allowed(PowerState::D0, PowerState::D1));
        assert!(power_transition_allowed(PowerState::D3, PowerState::D2));
        assert!(!power_transition_allowed(PowerState::D0, PowerState::D3));
    }
}
