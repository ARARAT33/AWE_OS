#![no_std]

use crate::arch::x86_64::io_out8;

/// Legacy PIT channel-0 programming used as the first deterministic kernel
/// time source. The interrupt path can later be replaced by LAPIC/HPET without
/// changing the scheduler-facing clock interface.
pub struct Pit {
    frequency_hz: u32,
}

impl Pit {
    pub const PIT_BASE_HZ: u32 = 1_193_182;

    pub const fn new(frequency_hz: u32) -> Option<Self> {
        if frequency_hz == 0 || frequency_hz > Self::PIT_BASE_HZ {
            None
        } else {
            Some(Self { frequency_hz })
        }
    }

    pub const fn frequency_hz(&self) -> u32 {
        self.frequency_hz
    }

    /// Program channel 0, mode 2 (rate generator), squarely suitable for a
    /// periodic scheduler tick. This only performs hardware I/O on x86_64.
    pub unsafe fn program(&self) {
        let divisor = (Self::PIT_BASE_HZ / self.frequency_hz).clamp(1, u16::MAX as u32) as u16;
        io_out8(0x43, 0x34);
        io_out8(0x40, divisor as u8);
        io_out8(0x40, (divisor >> 8) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_frequency() {
        assert!(Pit::new(0).is_none());
        assert!(Pit::new(100).is_some());
        assert!(Pit::new(Pit::PIT_BASE_HZ + 1).is_none());
    }
}
