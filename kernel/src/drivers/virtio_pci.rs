#![no_std]

use super::virtio::{VirtioFeatures, VirtioTransportState};

pub const PCI_VENDOR_ANY: u16 = 0xffff;
pub const PCI_DEVICE_ANY: u16 = 0xffff;
pub const VIRTIO_VENDOR_ID: u16 = 0x1af4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PciId {
    pub vendor: u16,
    pub device: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
}

impl PciId {
    pub const fn matches(self, vendor: u16, device: u16) -> bool {
        (vendor == PCI_VENDOR_ANY || self.vendor == vendor)
            && (device == PCI_DEVICE_ANY || self.device == device)
    }

    pub const fn is_virtio(self) -> bool {
        self.vendor == VIRTIO_VENDOR_ID
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bar {
    pub base: u64,
    pub size: u64,
    pub is_io: bool,
    pub prefetchable: bool,
}

impl Bar {
    pub fn validate(self) -> Result<(), PciError> {
        if self.size == 0 || !self.size.is_power_of_two() {
            return Err(PciError::InvalidBar);
        }
        if self.base & (self.size - 1) != 0 {
            return Err(PciError::InvalidBar);
        }
        if self.is_io && self.base > u32::MAX as u64 {
            return Err(PciError::InvalidBar);
        }
        Ok(())
    }

    pub fn end(self) -> Result<u64, PciError> {
        self.base
            .checked_add(self.size)
            .ok_or(PciError::AddressOverflow)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PciError {
    InvalidBar,
    AddressOverflow,
    InvalidConfigOffset,
    NotVirtio,
    MissingCommonCapability,
    MissingNotifyCapability,
    UnsupportedTransport,
    InvalidQueueCount,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtioPciCapabilities {
    pub common_cfg: Bar,
    pub notify_cfg: Bar,
    pub device_cfg: Option<Bar>,
    pub is_modern: bool,
}

impl VirtioPciCapabilities {
    pub fn validate(self) -> Result<(), PciError> {
        if !self.is_modern {
            return Err(PciError::UnsupportedTransport);
        }
        self.common_cfg.validate()?;
        self.notify_cfg.validate()?;
        if self.device_cfg.is_some() {
            // Device-specific config is optional, but when present it must be a
            // real MMIO/IO window just like the mandatory capabilities.
            match self.device_cfg.unwrap().validate() {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtioPciTransport {
    pub id: PciId,
    pub caps: VirtioPciCapabilities,
    pub transport: VirtioTransportState,
    queue_limit: u16,
}

impl VirtioPciTransport {
    pub fn new(
        id: PciId,
        caps: VirtioPciCapabilities,
        device_features: VirtioFeatures,
        queue_limit: u16,
    ) -> Result<Self, PciError> {
        if !id.is_virtio() {
            return Err(PciError::NotVirtio);
        }
        caps.validate()?;
        if queue_limit == 0 {
            return Err(PciError::InvalidQueueCount);
        }
        Ok(Self {
            id,
            caps,
            transport: VirtioTransportState::new(device_features),
            queue_limit,
        })
    }

    pub const fn queue_limit(&self) -> u16 {
        self.queue_limit
    }

    pub fn negotiate(
        &mut self,
        driver_features: VirtioFeatures,
    ) -> Result<VirtioFeatures, PciError> {
        self.transport
            .acknowledge()
            .map_err(|_| PciError::UnsupportedTransport)?;
        self.transport
            .set_driver()
            .map_err(|_| PciError::UnsupportedTransport)?;
        self.transport
            .negotiate(driver_features)
            .map_err(|_| PciError::UnsupportedTransport)
    }

    pub fn configure_queue_count(&mut self, count: u16) -> Result<(), PciError> {
        if count == 0 || count > self.queue_limit {
            return Err(PciError::InvalidQueueCount);
        }
        self.transport
            .configure_queues(count)
            .map_err(|_| PciError::InvalidQueueCount)
    }

    pub fn ready(&mut self) -> Result<(), PciError> {
        self.transport
            .driver_ok()
            .map_err(|_| PciError::UnsupportedTransport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn virtio_id() -> PciId {
        PciId {
            vendor: VIRTIO_VENDOR_ID,
            device: 0x1041,
            class: 0x02,
            subclass: 0,
            prog_if: 0,
        }
    }

    const fn caps() -> VirtioPciCapabilities {
        VirtioPciCapabilities {
            common_cfg: Bar {
                base: 0x1000,
                size: 0x1000,
                is_io: false,
                prefetchable: false,
            },
            notify_cfg: Bar {
                base: 0x2000,
                size: 0x1000,
                is_io: false,
                prefetchable: false,
            },
            device_cfg: None,
            is_modern: true,
        }
    }

    #[test]
    fn rejects_non_virtio_device() {
        let id = PciId {
            vendor: 0x1234,
            ..virtio_id()
        };
        assert_eq!(
            VirtioPciTransport::new(id, caps(), VirtioFeatures::VERSION_1, 8),
            Err(PciError::NotVirtio)
        );
    }

    #[test]
    fn validates_bar_alignment_and_overflow() {
        assert!(
            Bar {
                base: 0x1000,
                size: 0x1000,
                is_io: false,
                prefetchable: false
            }
            .validate()
            .is_ok()
        );
        assert_eq!(
            Bar {
                base: 0x1800,
                size: 0x1000,
                is_io: false,
                prefetchable: false
            }
            .validate(),
            Err(PciError::InvalidBar)
        );
        assert_eq!(
            Bar {
                base: u64::MAX - 3,
                size: 8,
                is_io: false,
                prefetchable: false
            }
            .end(),
            Err(PciError::AddressOverflow)
        );
    }

    #[test]
    fn modern_virtio_reaches_driver_ready() {
        let mut t = VirtioPciTransport::new(
            virtio_id(),
            caps(),
            VirtioFeatures::VERSION_1 | VirtioFeatures::RING_INDIRECT_DESC,
            8,
        )
        .unwrap();
        let negotiated = t
            .negotiate(VirtioFeatures::VERSION_1 | VirtioFeatures::RING_INDIRECT_DESC)
            .unwrap();
        assert!(negotiated.contains(VirtioFeatures::VERSION_1));
        t.configure_queue_count(2).unwrap();
        t.ready().unwrap();
    }

    #[test]
    fn queue_limit_is_hard_bounded() {
        let mut t =
            VirtioPciTransport::new(virtio_id(), caps(), VirtioFeatures::VERSION_1, 4).unwrap();
        t.negotiate(VirtioFeatures::VERSION_1).unwrap();
        assert_eq!(t.configure_queue_count(5), Err(PciError::InvalidQueueCount));
    }
}
