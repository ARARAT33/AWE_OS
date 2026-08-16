#![no_std]

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VirtioFeatures(pub u64);

impl VirtioFeatures {
    pub const VERSION_1: Self = Self(1 << 32);
    pub const ACCESS_PLATFORM: Self = Self(1 << 33);
    pub const RING_INDIRECT_DESC: Self = Self(1 << 28);
    pub const RING_EVENT_IDX: Self = Self(1 << 29);

    pub const fn contains(self, other: Self) -> bool { (self.0 & other.0) == other.0 }
    pub const fn intersection(self, other: Self) -> Self { Self(self.0 & other.0) }
}

pub struct VirtioDevice {
    negotiated: VirtioFeatures,
    ready: bool,
}

impl VirtioDevice {
    pub const fn new() -> Self { Self { negotiated: VirtioFeatures(0), ready: false } }

    /// Negotiate only features supported by both host and guest.
    pub const fn negotiate(device: VirtioFeatures, driver: VirtioFeatures) -> VirtioFeatures {
        device.intersection(driver)
    }

    pub fn initialize(&mut self, device: VirtioFeatures, driver: VirtioFeatures) -> bool {
        let negotiated = Self::negotiate(device, driver);
        if !negotiated.contains(VirtioFeatures::VERSION_1) { return false; }
        self.negotiated = negotiated;
        self.ready = true;
        true
    }

    pub const fn is_ready(&self) -> bool { self.ready }
    pub const fn features(&self) -> VirtioFeatures { self.negotiated }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiation_requires_common_features() {
        let host = VirtioFeatures(VirtioFeatures::VERSION_1.0 | VirtioFeatures::RING_EVENT_IDX.0);
        let guest = VirtioFeatures(VirtioFeatures::VERSION_1.0 | VirtioFeatures::RING_INDIRECT_DESC.0);
        let mut dev = VirtioDevice::new();
        assert!(dev.initialize(host, guest));
        assert!(dev.features().contains(VirtioFeatures::VERSION_1));
        assert!(!dev.features().contains(VirtioFeatures::RING_INDIRECT_DESC));
    }

    #[test]
    fn legacy_only_device_is_rejected() {
        let mut dev = VirtioDevice::new();
        assert!(!dev.initialize(VirtioFeatures(1), VirtioFeatures::VERSION_1));
        assert!(!dev.is_ready());
    }
}
