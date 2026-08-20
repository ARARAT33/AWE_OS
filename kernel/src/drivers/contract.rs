#![no_std]

use super::bus::DeviceId;
use super::universal::{DriverAbi, DriverOs};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MmioRegion {
    pub base: u64,
    pub length: u64,
}
impl MmioRegion {
    pub const fn end(&self) -> Option<u64> {
        self.base.checked_add(self.length)
    }
    pub const fn contains(&self, address: u64) -> bool {
        match self.end() {
            Some(end) => address >= self.base && address < end,
            None => false,
        }
    }
    pub const fn contains_range(&self, base: u64, length: u64) -> bool {
        if length == 0 {
            return false;
        }
        match base.checked_add(length) {
            Some(end) => match self.end() {
                Some(region_end) => base >= self.base && end <= region_end,
                None => false,
            },
            None => false,
        }
    }
    pub const fn overlaps(&self, other: &Self) -> bool {
        ranges_overlap(self.base, self.length, other.base, other.length)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterruptMode {
    None = 0,
    Legacy = 1,
    Msi = 2,
    MsiX = 3,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DmaPolicy {
    pub max_bytes: u64,
    pub address_bits: u8,
    pub coherent: bool,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DeviceContract<const M: usize> {
    pub vendor: u16,
    pub device: u16,
    pub class_code: u32,
    pub mmio: [Option<MmioRegion>; M],
    pub interrupt: InterruptMode,
    pub dma: DmaPolicy,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HardwareResource {
    pub mmio_base: u64,
    pub mmio_length: u64,
    pub dma_mask: u64,
    pub irq: u32,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverIdentity {
    pub os: DriverOs,
    pub abi: DriverAbi,
    pub api_version: u32,
    pub signed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContractValidationError {
    InvalidIdentity,
    InvalidHardware,
}

pub fn validate_identity(identity: DriverIdentity) -> Result<(), ContractValidationError> {
    if !identity.signed || identity.api_version == 0 {
        return Err(ContractValidationError::InvalidIdentity);
    }
    let compatible = match identity.os {
        DriverOs::Linux => matches!(
            identity.abi,
            DriverAbi::LinuxKmod | DriverAbi::LinuxUserMode | DriverAbi::Generic
        ),
        DriverOs::Android => matches!(identity.abi, DriverAbi::AndroidHal | DriverAbi::Generic),
        DriverOs::Windows => matches!(identity.abi, DriverAbi::WindowsCompat | DriverAbi::Generic),
        DriverOs::AweNative => matches!(identity.abi, DriverAbi::Native | DriverAbi::Generic),
        DriverOs::Bsd | DriverOs::Generic => true,
    };
    if compatible {
        Ok(())
    } else {
        Err(ContractValidationError::InvalidIdentity)
    }
}

pub fn validate_hardware(
    device: DeviceId,
    resources: HardwareResource,
) -> Result<(), ContractValidationError> {
    if !device.valid()
        || resources.mmio_length == 0
        || resources
            .mmio_base
            .checked_add(resources.mmio_length)
            .is_none()
        || resources.irq == u32::MAX
    {
        return Err(ContractValidationError::InvalidHardware);
    }
    Ok(())
}

impl<const M: usize> DeviceContract<M> {
    pub const fn valid(&self) -> bool {
        if self.vendor == 0
            || self.vendor == 0xffff
            || self.device == 0
            || self.device == 0xffff
            || self.dma.max_bytes == 0
            || self.dma.address_bits < 32
            || self.dma.address_bits > 64
        {
            return false;
        }
        let mut i = 0;
        while i < M {
            if let Some(r) = self.mmio[i] {
                if r.length == 0 || r.end().is_none() {
                    return false;
                }
                let mut j = 0;
                while j < i {
                    if let Some(other) = self.mmio[j]
                        && r.overlaps(&other) {
                            return false;
                        }
                    j += 1;
                }
            }
            i += 1;
        }
        true
    }
    pub const fn allows_mmio(&self, address: u64) -> bool {
        let mut i = 0;
        while i < M {
            if let Some(r) = self.mmio[i]
                && r.contains(address) {
                    return true;
                }
            i += 1;
        }
        false
    }
    pub const fn allows_mmio_range(&self, base: u64, length: u64) -> bool {
        let mut i = 0;
        while i < M {
            if let Some(r) = self.mmio[i]
                && r.contains_range(base, length) {
                    return true;
                }
            i += 1;
        }
        false
    }
    pub const fn allows_dma(&self, bytes: u64, address_bits: u8) -> bool {
        bytes != 0
            && bytes <= self.dma.max_bytes
            && address_bits >= 32
            && address_bits <= self.dma.address_bits
    }
    pub const fn allows_dma_range(&self, address: u64, bytes: u64) -> bool {
        if bytes == 0 || bytes > self.dma.max_bytes {
            return false;
        }
        let end = match address.checked_add(bytes) {
            Some(v) => v,
            None => return false,
        };
        let limit = if self.dma.address_bits == 64 {
            u64::MAX
        } else {
            1u64 << self.dma.address_bits
        };
        end <= limit
    }
    pub const fn allows_interrupt(&self, mode: InterruptMode) -> bool {
        matches!(
            (self.interrupt, mode),
            (InterruptMode::None, InterruptMode::None)
                | (InterruptMode::Legacy, InterruptMode::Legacy)
                | (InterruptMode::Msi, InterruptMode::Msi)
                | (InterruptMode::MsiX, InterruptMode::MsiX)
        )
    }
}

const fn ranges_overlap(a: u64, al: u64, b: u64, bl: u64) -> bool {
    let ae = match a.checked_add(al) {
        Some(v) => v,
        None => return true,
    };
    let be = match b.checked_add(bl) {
        Some(v) => v,
        None => return true,
    };
    a < be && b < ae
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c() -> DeviceContract<2> {
        DeviceContract {
            vendor: 1,
            device: 2,
            class_code: 0,
            mmio: [
                Some(MmioRegion {
                    base: 0x1000,
                    length: 0x1000,
                }),
                None,
            ],
            interrupt: InterruptMode::Msi,
            dma: DmaPolicy {
                max_bytes: 4096,
                address_bits: 48,
                coherent: true,
            },
        }
    }
    #[test]
    fn range_dma_interrupt_policies_work() {
        let x = c();
        assert!(x.valid());
        assert!(x.allows_mmio_range(0x1100, 0x100));
        assert!(!x.allows_mmio_range(0x1f00, 0x200));
        assert!(x.allows_dma(4096, 48));
        assert!(!x.allows_dma(4097, 48));
        assert!(x.allows_dma_range(0x1000, 4096));
        assert!(!x.allows_dma_range((1u64 << 48) - 1024, 2048));
        assert!(x.allows_interrupt(InterruptMode::Msi));
        assert!(!x.allows_interrupt(InterruptMode::Legacy))
    }
}
