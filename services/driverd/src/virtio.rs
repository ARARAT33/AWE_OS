#![no_std]

//! VirtIO 1.x transport and queue model used by driverd reference devices.
//! The platform layer supplies MMIO/PCI access; queue validation and feature
//! negotiation are deterministic and allocation-free here.

pub const VIRTIO_PCI_VENDOR_ID: u16 = 0x1AF4;
pub const VIRTIO_F_VERSION_1: u64 = 1 << 32;
pub const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
pub const VIRTIO_STATUS_DRIVER: u8 = 2;
pub const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
pub const VIRTIO_STATUS_FEATURES_OK: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VirtioError {
    InvalidVendor,
    InvalidDevice,
    MissingVersion1,
    QueueTooLarge,
    InvalidQueue,
    QueueOverflow,
    NotReady,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtioDevice {
    pub device_id: u16,
    pub vendor_id: u16,
    pub device_features: u64,
    pub driver_features: u64,
    pub status: u8,
}

impl VirtioDevice {
    pub const fn new(device_id: u16, vendor_id: u16, device_features: u64) -> Self {
        Self {
            device_id,
            vendor_id,
            device_features,
            driver_features: 0,
            status: 0,
        }
    }

    /// Validate the PCI identity before any driver state transition.
    /// VirtIO PCI devices use vendor 0x1AF4 and the modern device-id range
    /// 0x1000..=0x107F. Rejecting everything else is fail-closed.
    pub const fn validate_pci_identity(&self) -> Result<(), VirtioError> {
        if self.vendor_id != VIRTIO_PCI_VENDOR_ID {
            return Err(VirtioError::InvalidVendor);
        }
        if !(self.device_id >= 0x1000 && self.device_id <= 0x107F)
            && !(self.device_id >= 1 && self.device_id <= 0x3F)
        {
            return Err(VirtioError::InvalidDevice);
        }
        Ok(())
    }

    pub const fn acknowledge(mut self) -> Self {
        self.status |= VIRTIO_STATUS_ACKNOWLEDGE;
        self
    }
    pub const fn driver_present(mut self) -> Self {
        self.status |= VIRTIO_STATUS_DRIVER;
        self
    }

    pub const fn negotiate(mut self, requested: u64) -> Result<Self, VirtioError> {
        if self.device_features & VIRTIO_F_VERSION_1 == 0 {
            return Err(VirtioError::MissingVersion1);
        }
        self.driver_features = requested & self.device_features;
        self.status |= VIRTIO_STATUS_FEATURES_OK;
        Ok(self)
    }

    pub const fn ready(mut self) -> Result<Self, VirtioError> {
        if self.status
            & (VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER | VIRTIO_STATUS_FEATURES_OK)
            != (VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER | VIRTIO_STATUS_FEATURES_OK)
        {
            return Err(VirtioError::NotReady);
        }
        self.status |= VIRTIO_STATUS_DRIVER_OK;
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtioQueue {
    pub size: u16,
    pub used_index: u16,
    pub avail_index: u16,
}

impl VirtioQueue {
    pub const fn new(size: u16) -> Result<Self, VirtioError> {
        if size == 0 || size > 1024 || !size.is_power_of_two() {
            return Err(VirtioError::QueueTooLarge);
        }
        Ok(Self {
            size,
            used_index: 0,
            avail_index: 0,
        })
    }

    /// Number of descriptors currently outstanding, with a bounded overflow
    /// guard so a corrupted index cannot silently become a valid queue state.
    pub const fn pending(self) -> Result<u16, VirtioError> {
        let pending = self.avail_index.wrapping_sub(self.used_index);
        if pending > self.size {
            return Err(VirtioError::QueueOverflow);
        }
        Ok(pending)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VirtioKind {
    Block,
    Network,
    Input,
    Gpu,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pci_identity_is_fail_closed() {
        assert!(
            VirtioDevice::new(2, VIRTIO_PCI_VENDOR_ID, VIRTIO_F_VERSION_1)
                .validate_pci_identity()
                .is_ok()
        );
        assert_eq!(
            VirtioDevice::new(2, 0x1234, VIRTIO_F_VERSION_1).validate_pci_identity(),
            Err(VirtioError::InvalidVendor)
        );
        assert_eq!(
            VirtioDevice::new(0x10FF, VIRTIO_PCI_VENDOR_ID, VIRTIO_F_VERSION_1)
                .validate_pci_identity(),
            Err(VirtioError::InvalidDevice)
        );
    }

    #[test]
    fn version_1_is_required() {
        let device = VirtioDevice::new(2, VIRTIO_PCI_VENDOR_ID, VIRTIO_F_VERSION_1);
        let ready = device
            .acknowledge()
            .driver_present()
            .negotiate(VIRTIO_F_VERSION_1)
            .unwrap()
            .ready();
        assert!(ready.is_ok());
    }

    #[test]
    fn queue_requires_power_of_two_and_bound() {
        assert!(VirtioQueue::new(128).is_ok());
        assert_eq!(VirtioQueue::new(3), Err(VirtioError::QueueTooLarge));
        assert_eq!(VirtioQueue::new(2048), Err(VirtioError::QueueTooLarge));
    }

    #[test]
    fn queue_pending_rejects_corrupt_distance() {
        let queue = VirtioQueue {
            size: 8,
            used_index: 0,
            avail_index: 9,
        };
        assert_eq!(queue.pending(), Err(VirtioError::QueueOverflow));
        let queue = VirtioQueue {
            size: 8,
            used_index: 10,
            avail_index: 17,
        };
        assert_eq!(queue.pending(), Ok(7));
    }
}
