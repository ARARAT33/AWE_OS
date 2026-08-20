#![no_std]

/// Maximum number of PCI functions inspected by one bounded scan.
pub const MAX_PCI_FUNCTIONS: usize = 32;
/// Maximum buses inspected by the generic discovery pass.
pub const MAX_PCI_BUSES: usize = 8;
/// PCI configuration mechanism #1 I/O address port.
pub const CONFIG_ADDRESS_PORT: u16 = 0x0cf8;
/// PCI configuration mechanism #1 I/O data port.
pub const CONFIG_DATA_PORT: u16 = 0x0cfc;

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
pub struct PciConfigAddress {
    pub raw: u32,
}

impl PciConfigAddress {
    /// Build a PCI configuration-space address for mechanism #1.
    ///
    /// The returned value is suitable for the 0xCF8/0xCFC access contract;
    /// this layer intentionally does not perform port I/O itself.
    pub const fn new(bus: u8, device: u8, function: u8, offset: u8) -> Result<Self, PciError> {
        if device >= 32 || function >= 8 || offset & 0x03 != 0 {
            return Err(PciError::InvalidFunction);
        }
        let raw = 0x8000_0000u32
            | ((bus as u32) << 16)
            | ((device as u32) << 11)
            | ((function as u32) << 8)
            | offset as u32;
        Ok(Self { raw })
    }

    pub const fn bus(self) -> u8 { ((self.raw >> 16) & 0xff) as u8 }
    pub const fn device(self) -> u8 { ((self.raw >> 11) & 0x1f) as u8 }
    pub const fn function(self) -> u8 { ((self.raw >> 8) & 0x07) as u8 }
    pub const fn offset(self) -> u8 { (self.raw & 0xfc) as u8 }
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

pub struct PortConfigSpace;

impl ConfigSpace for PortConfigSpace {
    fn read32(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> Result<u32, PciError> {
        let addr = PciConfigAddress::new(bus, device, function, offset)?;
        unsafe {
            use crate::arch::x86_64::{io_in32, io_out32};
            io_out32(CONFIG_ADDRESS_PORT, addr.raw);
            Ok(io_in32(CONFIG_DATA_PORT))
        }
    }
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

    /// Scan a bounded bus range into a caller-owned fixed-capacity buffer.
    ///
    /// This is the runtime-neutral discovery primitive: a platform-specific
    /// `ConfigSpace` implementation supplies real config-space reads, while
    /// this layer guarantees deterministic bounds and no dynamic allocation.
    pub fn scan_buses<const N: usize>(
        &mut self,
        first_bus: u8,
        bus_count: u8,
        out: &mut [Option<PciFunction>; N],
    ) -> Result<usize, PciError> {
        if N > MAX_PCI_FUNCTIONS || bus_count as usize > MAX_PCI_BUSES {
            return Err(PciError::CapacityExceeded);
        }
        let end = (first_bus as u16) + (bus_count as u16);
        if end > 256 {
            return Err(PciError::InvalidFunction);
        }

        let mut found = 0usize;
        let mut bus = first_bus;
        while (bus as u16) < end {
            if found == N {
                break;
            }
            let mut device = 0u8;
            while device < 32 && found < N {
                let mut function = 0u8;
                while function < 8 && found < N {
                    if let Some(entry) = self.probe_function(bus, device, function)? {
                        out[found] = Some(entry);
                        found += 1;
                    }
                    function += 1;
                }
                device += 1;
            }
            bus = bus.wrapping_add(1);
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
    fn config_address_round_trips_bdf() {
        let a = PciConfigAddress::new(3, 5, 2, 0x14).unwrap();
        assert_eq!(a.bus(), 3);
        assert_eq!(a.device(), 5);
        assert_eq!(a.function(), 2);
        assert_eq!(a.offset(), 0x14);
        assert_eq!(a.raw, 0x8003_2a14);
    }

    #[test]
    fn config_address_rejects_invalid_alignment_and_fields() {
        assert_eq!(PciConfigAddress::new(0, 0, 0, 0x02), Err(PciError::InvalidFunction));
        assert_eq!(PciConfigAddress::new(0, 32, 0, 0), Err(PciError::InvalidFunction));
        assert_eq!(PciConfigAddress::new(0, 0, 8, 0), Err(PciError::InvalidFunction));
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
    fn multi_bus_scan_is_bounded_and_deterministic() {
        let mut e = Enumerator::new(FakeConfig { present: true, reads: 0 });
        let mut out = [None; 2];
        assert_eq!(e.scan_buses(1, 2, &mut out).unwrap(), 2);
        assert_eq!(out[0].unwrap().bus, 1);
        assert_eq!(out[1].unwrap().bus, 1);
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

    #[test]
    fn rejects_unbounded_bus_scan() {
        let mut e = Enumerator::new(FakeConfig { present: true, reads: 0 });
        let mut out = [None; 1];
        assert_eq!(e.scan_buses(0, (MAX_PCI_BUSES + 1) as u8, &mut out), Err(PciError::CapacityExceeded));
    }
}
