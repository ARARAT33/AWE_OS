#![no_std]

use super::core::{CoreError, HardwareAbstraction, HardwareInfo};

pub struct RegisterBank<const N: usize> {
    base: u64,
    words: [u32; N],
    irq_pending: bool,
    dma_submissions: u32,
}

impl<const N: usize> RegisterBank<N> {
    pub const fn new(base: u64) -> Self {
        Self {
            base,
            words: [0; N],
            irq_pending: false,
            dma_submissions: 0,
        }
    }
    fn index(&self, hw: &HardwareInfo, offset: u64) -> Result<usize, CoreError> {
        if self.base != hw.mmio_base || offset % 4 != 0 || !hw.mmio_contains(offset, 4) {
            return Err(CoreError::MmioDenied);
        }
        let index = (offset / 4) as usize;
        if index >= N {
            return Err(CoreError::MmioDenied);
        }
        Ok(index)
    }
    pub fn set_irq_pending(&mut self) {
        self.irq_pending = true;
    }
    pub const fn irq_pending(&self) -> bool {
        self.irq_pending
    }
    pub const fn dma_submissions(&self) -> u32 {
        self.dma_submissions
    }
}

impl<const N: usize> HardwareAbstraction for RegisterBank<N> {
    fn mmio_read32(&self, hw: &HardwareInfo, offset: u64) -> Result<u32, CoreError> {
        Ok(self.words[self.index(hw, offset)?])
    }
    fn mmio_write32(
        &mut self,
        hw: &HardwareInfo,
        offset: u64,
        value: u32,
    ) -> Result<(), CoreError> {
        let index = self.index(hw, offset)?;
        self.words[index] = value;
        Ok(())
    }
    fn irq_ack(&mut self, hw: &HardwareInfo) -> Result<(), CoreError> {
        if hw.irq == u32::MAX || !self.irq_pending {
            return Err(CoreError::IrqDenied);
        }
        self.irq_pending = false;
        Ok(())
    }
    fn dma_submit(&mut self, hw: &HardwareInfo, bytes: u64) -> Result<(), CoreError> {
        if bytes == 0 || bytes > hw.mmio_length || hw.dma_bits < 32 || hw.dma_bits > 64 {
            return Err(CoreError::DmaDenied);
        }
        self.dma_submissions = self.dma_submissions.saturating_add(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::bus::DeviceId;
    use super::*;
    fn hw() -> HardwareInfo {
        HardwareInfo {
            id: DeviceId {
                vendor: 1,
                device: 2,
                class: 0,
                revision: 1,
            },
            mmio_base: 0x1000,
            mmio_length: 0x100,
            irq: 5,
            dma_bits: 64,
        }
    }
    #[test]
    fn register_bank_round_trip() {
        let mut b: RegisterBank<8> = RegisterBank::new(0x1000);
        b.mmio_write32(&hw(), 0x10, 0xa5a5_55aa).unwrap();
        assert_eq!(b.mmio_read32(&hw(), 0x10), Ok(0xa5a5_55aa));
    }
    #[test]
    fn unaligned_and_foreign_base_are_rejected() {
        let b: RegisterBank<8> = RegisterBank::new(0x1000);
        assert_eq!(b.mmio_read32(&hw(), 0x11), Err(CoreError::MmioDenied));
        let foreign = HardwareInfo {
            mmio_base: 0x2000,
            ..hw()
        };
        assert_eq!(b.mmio_read32(&foreign, 0), Err(CoreError::MmioDenied));
    }
    #[test]
    fn irq_requires_pending_and_valid_line() {
        let mut b: RegisterBank<8> = RegisterBank::new(0x1000);
        assert_eq!(b.irq_ack(&hw()), Err(CoreError::IrqDenied));
        b.set_irq_pending();
        assert!(b.irq_ack(&hw()).is_ok());
        let mut bad = hw();
        bad.irq = u32::MAX;
        b.set_irq_pending();
        assert_eq!(b.irq_ack(&bad), Err(CoreError::IrqDenied));
    }
    #[test]
    fn dma_submission_is_counted() {
        let mut b: RegisterBank<8> = RegisterBank::new(0x1000);
        b.dma_submit(&hw(), 64).unwrap();
        assert_eq!(b.dma_submissions(), 1);
    }
}
