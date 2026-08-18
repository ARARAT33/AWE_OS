#![no_std]

/// Maximum number of PCI functions inspected by the bounded enumerator.
pub const MAX_PCI_FUNCTIONS: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PciFunction {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub bar0: u32,
    pub bar1: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PciError {
    InvalidFunction,
    ConfigReadFailed,
    CapacityExceeded,
}

/// Minimal, testable PCI config-space access contract.
pub trait ConfigSpace {
    fn read32(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> Result<u32, PciError>;
}

pub struct Enumerator<C> {
    config: C,
}

impl<C: ConfigSpace> Enumerator<C> {
    pub const fn new(config: C) -> Self {
        Self { config }
    }

    pub fn probe_function(
        &mut self,
        bus: u8,
        device: u8,
        function: u8,
    ) -> Result<Option<PciFunction>, PciError> {
        if device >= 32 || function >= 8 {
            return Err(PciError::InvalidFunction);
        }

        let id = self.config.read32(bus, device, function, 0x00)?;
        let vendor_id = (id & 0xffff) as u16;
        if vendor_id == 0xffff {
            return Ok(None);
        }
        let device_id = (id >> 16) as u16;

        let class = self.config.read32(bus, device, function, 0x08)?;
        let class_code = (class >> 24) as u8;
        let subclass = (class >> 16) as u8;
        let prog_if = (class >> 8) as u8;
        let bar0 = self.config.read32(bus, device, function, 0x10)?;
        let bar1 = self.config.read32(bus, device, function, 0x14)?;

        Ok(Some(PciFunction {
            bus,
            device,
            function,
            vendor_id,
            device_id,
            class_code,
            subclass,
            prog_if,
            bar0,
            bar1,
        }))
    }

    pub fn scan_bus<const N: usize>(
        &mut self,
        bus: u8,
        out: &mut [Option<PciFunction>; N],
    ) -> Result<usize, PciError> {
        if N > MAX_PCI_FUNCTIONS {
            return Err(PciError::CapacityExceeded);
        }
        let mut found = 0usize;
        let mut device = 0u8;
        while device < 32 {
            let mut function = 0u8;
            while function < 8 {
                if found == N {
                    return Ok(found);
                }
                if let Some(entry) = self.probe_function(bus, device, function)? {
                    out[found] = Some(entry);
                    found += 1;
                }
                function += 1;
            }
            device += 1;
        }
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeConfig {
        present: bool,
        reads: usize,
    }

    impl ConfigSpace for FakeConfig {
        fn read32(&mut self, _bus: u8, _device: u8, _function: u8, offset: u8) -> Result<u32, PciError> {
            self.reads += 1;
            if !self.present {
                return Ok(0xffff_ffff);
            }
            Ok(match offset {
                0x00 => 0x1000_1af4,
                0x08 => 0x0200_0000,
                0x10 => 0x0000_1000,
                0x14 => 0x0000_2000,
                _ => return Err(PciError::ConfigReadFailed),
            })
        }
    }

    #[test]
    fn absent_function_is_ignored() {
        let mut e = Enumerator::new(FakeConfig { present: false, reads: 0 });
        let mut out = [None; 1];
        assert_eq!(e.scan_bus(0, &mut out).unwrap(), 0);
        assert!(out[0].is_none());
    }

    #[test]
    fn present_function_is_decoded() {
        let mut e = Enumerator::new(FakeConfig { present: true, reads: 0 });
        let mut out = [None; 1];
        assert_eq!(e.scan_bus(0, &mut out).unwrap(), 1);
        let f = out[0].unwrap();
        assert_eq!(f.vendor_id, 0x1af4);
        assert_eq!(f.device_id, 0x1000);
        assert_eq!(f.class_code, 0x02);
        assert_eq!(f.bar0, 0x1000);
        assert_eq!(f.bar1, 0x2000);
    }

    #[test]
    fn rejects_invalid_function_numbers() {
        let mut e = Enumerator::new(FakeConfig { present: true, reads: 0 });
        assert_eq!(e.probe_function(0, 32, 0), Err(PciError::InvalidFunction));
        assert_eq!(e.probe_function(0, 0, 8), Err(PciError::InvalidFunction));
    }

    #[test]
    fn enforces_bounded_scan_output() {
        let mut e = Enumerator::new(FakeConfig { present: true, reads: 0 });
        let mut out = [None; MAX_PCI_FUNCTIONS + 1];
        assert_eq!(e.scan_bus(0, &mut out), Err(PciError::CapacityExceeded));
    }
}
