//! Built-in driver-service catalog.
//! The catalog is metadata only: implementations execute outside CellKernel.

use crate::{DriverClass, DriverDescriptor, DriverId, DriverState, DriverTrust};

pub const BUILTIN_DRIVER_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinDriver {
    pub id: DriverId,
    pub name: &'static str,
    pub class: DriverClass,
    pub abi_major: u16,
}

pub const BUILTIN_DRIVERS: [BuiltinDriver; BUILTIN_DRIVER_COUNT] = [
    BuiltinDriver { id: DriverId(1), name: "awe-pci", class: DriverClass::Other, abi_major: 1 },
    BuiltinDriver { id: DriverId(2), name: "awe-virtio", class: DriverClass::Virtio, abi_major: 1 },
    BuiltinDriver { id: DriverId(3), name: "awe-virtio-block", class: DriverClass::Storage, abi_major: 1 },
    BuiltinDriver { id: DriverId(4), name: "awe-virtio-net", class: DriverClass::Network, abi_major: 1 },
    BuiltinDriver { id: DriverId(5), name: "awe-display", class: DriverClass::Display, abi_major: 1 },
    BuiltinDriver { id: DriverId(6), name: "awe-input", class: DriverClass::Input, abi_major: 1 },
    BuiltinDriver { id: DriverId(7), name: "awe-audio", class: DriverClass::Audio, abi_major: 1 },
    BuiltinDriver { id: DriverId(8), name: "awe-linux-compat", class: DriverClass::Compatibility, abi_major: 1 },
    BuiltinDriver { id: DriverId(9), name: "awe-windows-compat", class: DriverClass::Compatibility, abi_major: 1 },
    BuiltinDriver { id: DriverId(10), name: "awe-android-compat", class: DriverClass::Compatibility, abi_major: 1 },
];

pub const fn descriptor(driver: BuiltinDriver) -> DriverDescriptor {
    DriverDescriptor {
        id: driver.id,
        class: driver.class,
        abi_major: driver.abi_major,
        abi_minor: 2,
        vendor: 0,
        device: 0,
        state: DriverState::Discovered,
        trust: DriverTrust::Verified,
    }
}

pub const fn descriptors() -> [DriverDescriptor; BUILTIN_DRIVER_COUNT] {
    [
        descriptor(BUILTIN_DRIVERS[0]), descriptor(BUILTIN_DRIVERS[1]),
        descriptor(BUILTIN_DRIVERS[2]), descriptor(BUILTIN_DRIVERS[3]),
        descriptor(BUILTIN_DRIVERS[4]), descriptor(BUILTIN_DRIVERS[5]),
        descriptor(BUILTIN_DRIVERS[6]), descriptor(BUILTIN_DRIVERS[7]),
        descriptor(BUILTIN_DRIVERS[8]), descriptor(BUILTIN_DRIVERS[9]),
    ]
}
