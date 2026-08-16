#![no_std]

use super::{DeviceContract, DeviceId, DriverBus};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DriverSource {
    NativeAwe = 0,
    LinuxPort = 1,
    AndroidPort = 2,
    WindowsPort = 3,
    BsdPort = 4,
    OtherPort = 5,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DriverManifest {
    pub vendor: u16,
    pub device: u16,
    pub class_code: u32,
    pub source: DriverSource,
    pub api_version: u16,
    pub verified: bool,
}

impl DriverManifest {
    pub const API_VERSION: u16 = 1;
    pub const fn matches(&self, id: DeviceId) -> bool {
        self.vendor == id.vendor && self.device == id.device && self.api_version == Self::API_VERSION
    }
    pub const fn can_load(&self) -> bool { self.verified }
}

pub struct CompatibilityRegistry<const N: usize> {
    entries: [Option<DriverManifest>; N],
}

impl<const N: usize> CompatibilityRegistry<N> {
    pub const fn new() -> Self { Self { entries: [None; N] } }
    pub fn register(&mut self, manifest: DriverManifest) -> bool {
        if !manifest.can_load() { return false; }
        let mut i = 0;
        while i < N {
            if self.entries[i].is_none() {
                self.entries[i] = Some(manifest);
                return true;
            }
            i += 1;
        }
        false
    }
    pub fn find(&self, id: DeviceId) -> Option<DriverManifest> {
        let mut i = 0;
        while i < N {
            if let Some(m) = self.entries[i] {
                if m.matches(id) { return Some(m); }
            }
            i += 1;
        }
        None
    }
}

pub const fn validate_contract(contract: DeviceContract) -> bool { contract.is_valid() }

pub fn bind_compatible_driver<const N: usize>(
    bus: &mut DriverBus<N>,
    manifest: DriverManifest,
    contract: DeviceContract,
) -> bool {
    manifest.can_load() && manifest.matches(contract.id) && validate_contract(contract) && bus.register(contract)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::{DeviceKind, DmaPolicy, InterruptMode, MmioRegion};

    fn contract() -> DeviceContract {
        DeviceContract::new(
            DeviceId::new(0x1af4, 0x1001), DeviceKind::VirtioBlock,
            MmioRegion::new(0x1000, 0x1000), DmaPolicy::new(0xffff_ffff, false), InterruptMode::Msi,
        )
    }

    #[test]
    fn only_verified_manifest_registers() {
        let mut r: CompatibilityRegistry<4> = CompatibilityRegistry::new();
        let good = DriverManifest { vendor: 0x1af4, device: 0x1001, class_code: 0, source: DriverSource::NativeAwe, api_version: 1, verified: true };
        assert!(!r.register(DriverManifest { verified: false, ..good }));
        assert!(r.register(good));
        assert!(r.find(DeviceId::new(0x1af4, 0x1001)).is_some());
    }

    #[test]
    fn binding_rejects_unverified_driver() {
        let mut bus: DriverBus<4> = DriverBus::new();
        let manifest = DriverManifest { vendor: 0x1af4, device: 0x1001, class_code: 0, source: DriverSource::LinuxPort, api_version: 1, verified: false };
        assert!(!bind_compatible_driver(&mut bus, manifest, contract()));
    }
}
