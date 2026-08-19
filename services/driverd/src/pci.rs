#![no_std]

//! Bounded PCI configuration-space enumeration and BAR validation used by
//! driverd. Hardware access is provided by a platform backend; enumeration
//! and resource validation are deterministic and allocation-free.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciLocation {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciDevice {
    pub location: PciLocation,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub header_type: u8,
    pub interrupt_line: u8,
}

pub trait PciConfigAccess {
    fn read_u32(&self, location: PciLocation, offset: u8) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PciBarKind {
    Memory32,
    Memory64,
    Io,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciBar {
    pub index: u8,
    pub kind: PciBarKind,
    pub base: u64,
    pub size: u64,
    pub prefetchable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PciBarError {
    InvalidIndex,
    InvalidSize,
    MisalignedBase,
    Overflow,
    Unsupported64BitPair,
}

/// Decodes a PCI BAR from its value and the probe mask returned after writing
/// all ones to the BAR. The caller owns the hardware write/read sequence.
pub fn decode_bar(
    index: u8,
    value: u32,
    probe: u32,
    upper: Option<(u32, u32)>,
) -> Result<PciBar, PciBarError> {
    if index >= 6 {
        return Err(PciBarError::InvalidIndex);
    }

    if (value & 1) != 0 {
        let base = (value & 0xFFFF_FFFC) as u64;
        let mask_32 = probe & 0xFFFF_FFFC;
        if mask_32 == 0 {
            return Err(PciBarError::InvalidSize);
        }
        let size = (!mask_32).wrapping_add(1) as u64;
        if size == 0 || !size.is_power_of_two() {
            return Err(PciBarError::InvalidSize);
        }
        if base % size != 0 {
            return Err(PciBarError::MisalignedBase);
        }
        return Ok(PciBar {
            index,
            kind: PciBarKind::Io,
            base,
            size,
            prefetchable: false,
        });
    }

    let mem_type = (value >> 1) & 0x3;
    let prefetchable = (value & 0x8) != 0;
    let low_base = (value & 0xFFFF_FFF0) as u64;
    let low_mask_32 = probe & 0xFFFF_FFF0;

    if low_mask_32 == 0 {
        return Err(PciBarError::InvalidSize);
    }

    let (base, mask) = match mem_type {
        0 => (low_base, (0xFFFF_FFFF_0000_0000u64 | (low_mask_32 as u64))),
        2 => {
            let (upper_value, upper_probe) = upper.ok_or(PciBarError::Unsupported64BitPair)?;
            (
                ((upper_value as u64) << 32) | low_base,
                ((upper_probe as u64) << 32) | (low_mask_32 as u64),
            )
        }
        _ => return Err(PciBarError::InvalidSize),
    };

    let size = (!mask).wrapping_add(1);
    if size == 0 || !size.is_power_of_two() {
        return Err(PciBarError::InvalidSize);
    }
    if base % size != 0 {
        return Err(PciBarError::MisalignedBase);
    }
    base.checked_add(size - 1).ok_or(PciBarError::Overflow)?;

    Ok(PciBar {
        index,
        kind: if mem_type == 2 {
            PciBarKind::Memory64
        } else {
            PciBarKind::Memory32
        },
        base,
        size,
        prefetchable,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PciError {
    Full,
}

pub struct PciDeviceTable<const N: usize> {
    entries: [Option<PciDevice>; N],
    len: usize,
}

impl<const N: usize> Default for PciDeviceTable<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PciDeviceTable<N> {
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, device: PciDevice) -> Result<(), PciError> {
        if self.len == N {
            return Err(PciError::Full);
        }
        self.entries[self.len] = Some(device);
        self.len += 1;
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<PciDevice> {
        if index >= self.len {
            None
        } else {
            self.entries[index]
        }
    }
}

pub fn enumerate<C: PciConfigAccess, const N: usize>(
    access: &C,
    table: &mut PciDeviceTable<N>,
) -> Result<(), PciError> {
    let mut bus = 0u16;
    while bus <= 0xFF {
        let mut slot = 0u8;
        while slot < 32 {
            let location0 = PciLocation {
                bus: bus as u8,
                device: slot,
                function: 0,
            };
            let id = access.read_u32(location0, 0x00);
            if (id & 0xFFFF) != 0xFFFF {
                let header = (access.read_u32(location0, 0x0C) >> 16) as u8;
                let multifunction = (header & 0x80) != 0;
                let functions = if multifunction { 8 } else { 1 };
                let mut function = 0u8;
                while function < functions {
                    let location = PciLocation {
                        bus: bus as u8,
                        device: slot,
                        function,
                    };
                    let vendor_device = access.read_u32(location, 0x00);
                    if (vendor_device & 0xFFFF) != 0xFFFF {
                        let class_word = access.read_u32(location, 0x08);
                        table.push(PciDevice {
                            location,
                            vendor_id: vendor_device as u16,
                            device_id: (vendor_device >> 16) as u16,
                            class: (class_word >> 24) as u8,
                            subclass: (class_word >> 16) as u8,
                            prog_if: (class_word >> 8) as u8,
                            header_type: (access.read_u32(location, 0x0C) >> 16) as u8,
                            interrupt_line: access.read_u32(location, 0x3C) as u8,
                        })?;
                    }
                    function += 1;
                }
            }
            slot += 1;
        }
        bus += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake;
    impl PciConfigAccess for Fake {
        fn read_u32(&self, location: PciLocation, offset: u8) -> u32 {
            if location.bus == 0 && location.device == 1 && location.function == 0 {
                match offset {
                    0x00 => 0x1234_5678,
                    0x08 => 0x0101_0203,
                    0x0C => 0,
                    0x3C => 5,
                    _ => 0,
                }
            } else {
                0xFFFF_FFFF
            }
        }
    }

    #[test]
    fn enumerates_present_device() {
        let mut table: PciDeviceTable<4> = PciDeviceTable::new();
        assert_eq!(enumerate(&Fake, &mut table), Ok(()));
        assert_eq!(table.len(), 1);
        let d = table.get(0).unwrap();
        assert_eq!(d.vendor_id, 0x5678);
        assert_eq!(d.device_id, 0x1234);
        assert_eq!(d.interrupt_line, 5);
    }

    #[test]
    fn decodes_32_bit_memory_bar() {
        let bar = decode_bar(0, 0x8000_0000, 0xFFFF_F000, None).unwrap();
        assert_eq!(bar.kind, PciBarKind::Memory32);
        assert_eq!(bar.base, 0x8000_0000);
        assert_eq!(bar.size, 0x1000);
    }

    #[test]
    fn decodes_64_bit_prefetchable_bar() {
        let bar = decode_bar(
            2,
            0x0000_000C,
            0xFFFF_F000,
            Some((0x0000_0001, 0xFFFF_FFFF)),
        )
        .unwrap();
        assert_eq!(bar.kind, PciBarKind::Memory64);
        assert!(bar.prefetchable);
        assert_eq!(bar.base, 0x0000_0001_0000_0000);
    }

    #[test]
    fn rejects_invalid_bar_size() {
        assert_eq!(
            decode_bar(0, 0x1000, 0, None),
            Err(PciBarError::InvalidSize)
        );
    }

    #[test]
    fn rejects_misaligned_bar() {
        assert_eq!(
            decode_bar(0, 0x1800, 0xFFFF_F000, None),
            Err(PciBarError::MisalignedBase)
        );
    }

    #[test]
    fn rejects_64_bit_bar_without_upper_pair() {
        assert_eq!(
            decode_bar(0, 0x0000_0004, 0xFFFF_F000, None),
            Err(PciBarError::Unsupported64BitPair)
        );
    }
}
