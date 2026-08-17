#![no_std]

//! AWE 61.5 canonical device/driver binding contract.
//!
//! This is deliberately hardware-neutral: PCI/ACPI/VirtIO enumeration and
//! concrete driver execution belong to the standalone driver service and later
//! 65% driver milestones.

use super::{DeviceClass, DeviceId};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchKind {
    Exact = 0,
    Class = 1,
    Fallback = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceMatch {
    pub vendor: u16,
    pub product: u16,
    pub class: DeviceClass,
    pub kind: MatchKind,
}

impl DeviceMatch {
    pub const fn exact(vendor: u16, product: u16, class: DeviceClass) -> Self {
        Self { vendor, product, class, kind: MatchKind::Exact }
    }

    pub const fn class(class: DeviceClass) -> Self {
        Self { vendor: 0, product: 0, class, kind: MatchKind::Class }
    }

    pub const fn matches(self, vendor: u16, product: u16, class: DeviceClass) -> bool {
        match self.kind {
            MatchKind::Exact => self.vendor == vendor && self.product == product && self.class == class,
            MatchKind::Class => self.class == class,
            MatchKind::Fallback => true,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceGrant {
    pub device: DeviceId,
    pub mmio_bytes: u64,
    pub io_bytes: u64,
    pub dma_bytes: u64,
    pub interrupt_count: u16,
}

impl ResourceGrant {
    pub const fn empty(device: DeviceId) -> Self {
        Self { device, mmio_bytes: 0, io_bytes: 0, dma_bytes: 0, interrupt_count: 0 }
    }

    pub const fn within(self, budget: ResourceGrant) -> bool {
        self.device == budget.device
            && self.mmio_bytes <= budget.mmio_bytes
            && self.io_bytes <= budget.io_bytes
            && self.dma_bytes <= budget.dma_bytes
            && self.interrupt_count <= budget.interrupt_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingDecision {
    Reject,
    Accept,
}

pub const fn decide_binding(
    matcher: DeviceMatch,
    vendor: u16,
    product: u16,
    class: DeviceClass,
    requested: ResourceGrant,
    allowed: ResourceGrant,
) -> BindingDecision {
    if !matcher.matches(vendor, product, class) || !requested.within(allowed) {
        BindingDecision::Reject
    } else {
        BindingDecision::Accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_is_strict() {
        let m = DeviceMatch::exact(0x1234, 0x5678, DeviceClass::Network);
        assert!(m.matches(0x1234, 0x5678, DeviceClass::Network));
        assert!(!m.matches(0x1234, 0x9999, DeviceClass::Network));
    }

    #[test]
    fn class_match_is_not_vendor_specific() {
        let m = DeviceMatch::class(DeviceClass::Storage);
        assert!(m.matches(1, 2, DeviceClass::Storage));
        assert!(!m.matches(1, 2, DeviceClass::Display));
    }

    #[test]
    fn resource_ownership_is_bounded() {
        let device = DeviceId(7);
        let budget = ResourceGrant { device, mmio_bytes: 4096, io_bytes: 128, dma_bytes: 8192, interrupt_count: 4 };
        let requested = ResourceGrant { device, mmio_bytes: 2048, io_bytes: 64, dma_bytes: 4096, interrupt_count: 2 };
        assert!(requested.within(budget));
        let too_much = ResourceGrant { device, mmio_bytes: 8192, io_bytes: 64, dma_bytes: 4096, interrupt_count: 2 };
        assert!(!too_much.within(budget));
    }

    #[test]
    fn binding_is_fail_closed() {
        let matcher = DeviceMatch::exact(1, 2, DeviceClass::Network);
        let allowed = ResourceGrant { device: DeviceId(9), mmio_bytes: 4096, io_bytes: 64, dma_bytes: 4096, interrupt_count: 2 };
        let requested = ResourceGrant { device: DeviceId(9), mmio_bytes: 8192, io_bytes: 64, dma_bytes: 4096, interrupt_count: 2 };
        assert_eq!(decide_binding(matcher, 1, 2, DeviceClass::Network, requested, allowed), BindingDecision::Reject);
    }
}
