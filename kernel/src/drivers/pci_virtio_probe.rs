#![no_std]

use super::pci::PciFunction;
use super::virtio::VirtioFeatures;
use super::virtio_pci::{
    Bar, PciError, PciId, VIRTIO_VENDOR_ID, VirtioPciCapabilities, VirtioPciTransport,
};

/// Maximum number of VirtIO functions admitted by one bounded probe pass.
pub const MAX_VIRTIO_PROBES: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VirtioDeviceKind {
    Unknown,
    Network,
    Block,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProbeError {
    NotVirtio,
    InvalidBar,
    UnsupportedTransport,
    CapacityExceeded,
}

impl From<PciError> for ProbeError {
    fn from(value: PciError) -> Self {
        match value {
            PciError::NotVirtio => Self::NotVirtio,
            PciError::InvalidBar | PciError::AddressOverflow => Self::InvalidBar,
            PciError::UnsupportedTransport
            | PciError::MissingCommonCapability
            | PciError::MissingNotifyCapability
            | PciError::InvalidConfigOffset => Self::UnsupportedTransport,
            PciError::InvalidQueueCount => Self::CapacityExceeded,
        }
    }
}

/// Pure admission/translation layer between bounded PCI enumeration and the
/// validated modern VirtIO PCI transport. It performs no MMIO writes; the
/// platform-specific register-access layer owns that step.
pub struct VirtioPciProbe;

impl VirtioPciProbe {
    pub const fn classify(function: &PciFunction) -> Result<VirtioDeviceKind, ProbeError> {
        if function.vendor_id != VIRTIO_VENDOR_ID {
            return Err(ProbeError::NotVirtio);
        }
        Ok(match function.device_id {
            0x1041 => VirtioDeviceKind::Network,
            0x1042 => VirtioDeviceKind::Block,
            _ => VirtioDeviceKind::Unknown,
        })
    }

    pub fn prepare(
        function: PciFunction,
        common_size: u64,
        notify_size: u64,
        device_size: Option<u64>,
        features: VirtioFeatures,
        queue_limit: u16,
    ) -> Result<(VirtioDeviceKind, VirtioPciTransport), ProbeError> {
        let kind = Self::classify(&function)?;
        let common_cfg = Self::bar_from_pci(function.bar0, common_size)?;
        let notify_cfg = Self::bar_from_pci(function.bar1, notify_size)?;
        let device_cfg = match device_size {
            Some(size) => Some(Self::bar_from_pci(function.bar1, size)?),
            None => None,
        };
        let caps = VirtioPciCapabilities {
            common_cfg,
            notify_cfg,
            device_cfg,
            is_modern: true,
        };
        let transport = VirtioPciTransport::new(function_id(function), caps, features, queue_limit)
            .map_err(ProbeError::from)?;
        Ok((kind, transport))
    }

    fn bar_from_pci(raw: u32, size: u64) -> Result<Bar, ProbeError> {
        if size == 0 {
            return Err(ProbeError::InvalidBar);
        }
        let is_io = (raw & 1) != 0;
        let base = if is_io {
            (raw & !0x3) as u64
        } else {
            (raw & !0xf) as u64
        };
        let bar = Bar {
            base,
            size,
            is_io,
            prefetchable: false,
        };
        match bar.validate() {
            Ok(()) => Ok(bar),
            Err(_) => Err(ProbeError::InvalidBar),
        }
    }
}

const fn function_id(function: PciFunction) -> PciId {
    PciId {
        vendor: function.vendor_id,
        device: function.device_id,
        class: function.class_code,
        subclass: function.subclass,
        prog_if: function.prog_if,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn network_function() -> PciFunction {
        PciFunction {
            bus: 0,
            device: 5,
            function: 0,
            vendor_id: VIRTIO_VENDOR_ID,
            device_id: 0x1041,
            class_code: 0x02,
            subclass: 0,
            prog_if: 0,
            bar0: 0x1000,
            bar1: 0x2000,
        }
    }

    #[test]
    fn classifies_supported_virtio_devices() {
        assert_eq!(
            VirtioPciProbe::classify(&network_function()),
            Ok(VirtioDeviceKind::Network)
        );
        let block = PciFunction {
            device_id: 0x1042,
            ..network_function()
        };
        assert_eq!(
            VirtioPciProbe::classify(&block),
            Ok(VirtioDeviceKind::Block)
        );
    }

    #[test]
    fn rejects_non_virtio_before_transport_creation() {
        let non_virtio = PciFunction {
            vendor_id: 0x1234,
            ..network_function()
        };
        assert_eq!(
            VirtioPciProbe::classify(&non_virtio),
            Err(ProbeError::NotVirtio)
        );
    }

    #[test]
    fn prepares_bounded_transport_from_enumerated_function() {
        let (kind, mut transport) = VirtioPciProbe::prepare(
            network_function(),
            0x1000,
            0x1000,
            None,
            VirtioFeatures::VERSION_1,
            4,
        )
        .unwrap();
        assert_eq!(kind, VirtioDeviceKind::Network);
        transport.negotiate(VirtioFeatures::VERSION_1).unwrap();
        transport.configure_queue_count(2).unwrap();
        transport.ready().unwrap();
    }

    #[test]
    fn rejects_zero_capability_window() {
        assert_eq!(
            VirtioPciProbe::prepare(
                network_function(),
                0,
                0x1000,
                None,
                VirtioFeatures::VERSION_1,
                4,
            ),
            Err(ProbeError::InvalidBar)
        );
    }

    #[test]
    fn rejects_unbounded_queue_configuration() {
        let result = VirtioPciProbe::prepare(
            network_function(),
            0x1000,
            0x1000,
            None,
            VirtioFeatures::VERSION_1,
            0,
        );
        assert_eq!(result, Err(ProbeError::CapacityExceeded));
    }
}
