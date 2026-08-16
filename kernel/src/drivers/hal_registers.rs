#![no_std]

use super::core::{CoreError, HardwareAbstraction, HardwareInfo};

pub struct RegisterBank<const N: usize> {
    base: u64,
    words: [u32; N],
    irq_pending: bool,
}

impl<const N: usize> RegisterBank<N> {
    pub const fn new(base: u64) -> Self {
        Self { base, words: [0; N], irq_pending: false }
    }

    fn index(&self, hw: &HardwareInfo, offset: u64) -> Result<usize, CoreError> {
        if self.base != hw.mmio_base || offset % 4 != 0 || !hw.mmio_contains(offset, 4) {
            return Err(CoreError::MmioDenied);
        }
        let index = usize::try_from(offset / 4).map_err(|_| CoreError::MmioDenied)?;
        if index >= N { return Err(CoreError::MmioDenied); }
        Ok(index)
    }

    pub fn set_irq_pending(&mut self) { self.irq_pending = true; }
    pub const fn irq_pending(&self) -> bool { self.irq_pending }
}

impl<const N: usize> HardwareAbstraction for RegisterBank<N> {
    fn mmio_read32(&self, hw: &HardwareInfo, offset: u64) -> Result<u32, CoreError> {
        Ok(self.words[self.index(hw, offset)?])
    }

    fn mmio_write32(&mut self, hw: &HardwareInfo, offset: u64, value: u32) -> Result<(), CoreError> {
        let index = self.index(hw, offset)?;
        self.words[index] = value;
        Ok(())
    }

    fn irq_ack(&mut self, _hw: &HardwareInfo) -> Result<(), CoreError> {
        if !self.irq_pending { return Err(CoreError::IrqDenied); }
        self.irq_pending = false;
        Ok(())
    }

    fn dma_submit(&mut self, hw: &HardwareInfo, bytes: u64) -> Result<(), CoreError> {
        if bytes == 0 || bytes > hw.mmio_length || hw.dma_bits < 32 || hw.dma_bits > 64 {
            return Err(CoreError::DmaDenied);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::bus::DeviceId;

    fn hw() -> HardwareInfo {
        HardwareInfo { id: DeviceId { vendor: 1, device: 2, class: 0, revision: 1 }, mmio_base: 0x1000, mmio_length: 0x100, irq: 5, dma_bits: 64 }
    }

    #[test]
    fn register_bank_round_trip() {
        let mut bank: RegisterBank<8> = RegisterBank::new(0x1000);
        bank.mmio_write32(&hw(), 0x10, 0xa5a5_55aa).unwrap();
        assert_eq!(bank.mmio_read32(&hw(), 0x10), Ok(0xa5a5_55aa));
    }

    #[test]
    fn unaligned_access_is_rejected() {
        let bank: RegisterBank<8> = RegisterBank::new(0x1000);
        assert_eq!(bank.mmio_read32(&hw(), 0x11), Err(CoreError::MmioDenied));
    }

    #[test]
    fn irq_requires_pending_state() {
        let mut bank: RegisterBank<8> = RegisterBank::new(0x1000);
        assert_eq!(bank.irq_ack(&hw()), Err(CoreError::IrqDenied));
        bank.set_irq_pending();
        assert!(bank.irq_ack(&hw()).is_ok());
        assert!(!bank.irq_pending());
    }
}
