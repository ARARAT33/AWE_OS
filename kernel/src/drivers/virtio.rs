#![no_std]

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VirtioFeatures(pub u64);

impl VirtioFeatures {
    pub const VERSION_1: Self = Self(1 << 32);
    pub const ACCESS_PLATFORM: Self = Self(1 << 33);
    pub const RING_INDIRECT_DESC: Self = Self(1 << 28);
    pub const RING_EVENT_IDX: Self = Self(1 << 29);
    pub const RING_PACKED: Self = Self(1 << 34);
    pub const fn contains(self, other: Self) -> bool { (self.0 & other.0) == other.0 }
    pub const fn intersection(self, other: Self) -> Self { Self(self.0 & other.0) }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VirtioError { MissingVersion, AlreadyReady, QueueCountZero, QueueTooLarge, InvalidQueueAlignment }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtioQueueConfig { pub size: u16, pub alignment: u16 }

impl VirtioQueueConfig {
    pub const fn validate(self, max_size: u16) -> Result<(), VirtioError> {
        if self.size == 0 { return Err(VirtioError::QueueCountZero); }
        if self.size > max_size { return Err(VirtioError::QueueTooLarge); }
        if self.alignment == 0 || !self.alignment.is_power_of_two() { return Err(VirtioError::InvalidQueueAlignment); }
        Ok(())
    }
}

pub struct VirtioDevice {
    negotiated: VirtioFeatures,
    ready: bool,
    queue_count: u16,
}

impl VirtioDevice {
    pub const fn new() -> Self { Self { negotiated: VirtioFeatures(0), ready: false, queue_count: 0 } }
    pub const fn negotiate(device: VirtioFeatures, driver: VirtioFeatures) -> VirtioFeatures { device.intersection(driver) }

    pub fn initialize(&mut self, device: VirtioFeatures, driver: VirtioFeatures) -> Result<(), VirtioError> {
        if self.ready { return Err(VirtioError::AlreadyReady); }
        let negotiated = Self::negotiate(device, driver);
        if !negotiated.contains(VirtioFeatures::VERSION_1) { return Err(VirtioError::MissingVersion); }
        self.negotiated = negotiated;
        Ok(())
    }

    pub fn configure_queues(&mut self, queues: &[VirtioQueueConfig], max_size: u16) -> Result<(), VirtioError> {
        if !self.negotiated.contains(VirtioFeatures::VERSION_1) { return Err(VirtioError::MissingVersion); }
        if queues.is_empty() { return Err(VirtioError::QueueCountZero); }
        for queue in queues.iter().copied() { queue.validate(max_size)?; }
        self.queue_count = queues.len() as u16;
        self.ready = true;
        Ok(())
    }

    pub const fn is_ready(&self) -> bool { self.ready }
    pub const fn features(&self) -> VirtioFeatures { self.negotiated }
    pub const fn queue_count(&self) -> u16 { self.queue_count }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn negotiation_requires_common_features() { let host=VirtioFeatures((1<<32)|(1<<29)); let guest=VirtioFeatures((1<<32)|(1<<28)); let mut dev=VirtioDevice::new(); dev.initialize(host,guest).unwrap(); assert!(dev.features().contains(VirtioFeatures::VERSION_1)); assert!(!dev.features().contains(VirtioFeatures::RING_INDIRECT_DESC)); }
    #[test] fn legacy_only_device_is_rejected() { let mut dev=VirtioDevice::new(); assert_eq!(dev.initialize(VirtioFeatures(1),VirtioFeatures::VERSION_1),Err(VirtioError::MissingVersion)); assert!(!dev.is_ready()); }
    #[test] fn queues_are_validated_before_ready() { let mut dev=VirtioDevice::new(); dev.initialize(VirtioFeatures::VERSION_1,VirtioFeatures::VERSION_1).unwrap(); assert_eq!(dev.configure_queues(&[],128),Err(VirtioError::QueueCountZero)); assert!(!dev.is_ready()); }
    #[test] fn valid_queue_configuration_makes_device_ready() { let mut dev=VirtioDevice::new(); dev.initialize(VirtioFeatures::VERSION_1,VirtioFeatures::VERSION_1).unwrap(); dev.configure_queues(&[VirtioQueueConfig{size:64,alignment:4096},VirtioQueueConfig{size:128,alignment:4096}],128).unwrap(); assert!(dev.is_ready()); assert_eq!(dev.queue_count(),2); }
    #[test] fn invalid_queue_does_not_partially_configure() { let mut dev=VirtioDevice::new(); dev.initialize(VirtioFeatures::VERSION_1,VirtioFeatures::VERSION_1).unwrap(); assert_eq!(dev.configure_queues(&[VirtioQueueConfig{size:64,alignment:3}],128),Err(VirtioError::InvalidQueueAlignment)); assert_eq!(dev.queue_count(),0); assert!(!dev.is_ready()); }
}
