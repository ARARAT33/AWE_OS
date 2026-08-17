#![no_std]

//! AWE 62.0 device/driver resource-and-capability boundary.
//!
//! Hardware discovery and concrete execution remain in `services/driverd`.
//! CellKernel owns only the identity, capability and bounded resource grant
//! contract needed to admit an already-discovered device to a driver service.

use super::{DeviceClass, DeviceId};
use crate::ipc::CapabilityHandle;
use crate::system_contract::{CapabilitySet, KernelCapability, ServiceId};

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
        Self {
            vendor,
            product,
            class,
            kind: MatchKind::Exact,
        }
    }

    pub const fn class(class: DeviceClass) -> Self {
        Self {
            vendor: 0,
            product: 0,
            class,
            kind: MatchKind::Class,
        }
    }

    pub const fn fallback() -> Self {
        Self {
            vendor: 0,
            product: 0,
            class: DeviceClass::Unknown,
            kind: MatchKind::Fallback,
        }
    }

    pub fn matches(self, vendor: u16, product: u16, class: DeviceClass) -> bool {
        match self.kind {
            MatchKind::Exact => {
                self.vendor == vendor && self.product == product && self.class == class
            }
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
        Self {
            device,
            mmio_bytes: 0,
            io_bytes: 0,
            dma_bytes: 0,
            interrupt_count: 0,
        }
    }

    pub fn within(self, budget: ResourceGrant) -> bool {
        self.device == budget.device
            && self.mmio_bytes <= budget.mmio_bytes
            && self.io_bytes <= budget.io_bytes
            && self.dma_bytes <= budget.dma_bytes
            && self.interrupt_count <= budget.interrupt_count
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverGrant {
    pub service: ServiceId,
    pub endpoint: CapabilityHandle,
    pub device: DeviceId,
    pub resources: ResourceGrant,
    pub capabilities: CapabilitySet,
}

impl DriverGrant {
    pub const fn new(
        service: ServiceId,
        endpoint: CapabilityHandle,
        device: DeviceId,
        resources: ResourceGrant,
        capabilities: CapabilitySet,
    ) -> Self {
        Self {
            service,
            endpoint,
            device,
            resources,
            capabilities,
        }
    }

    pub fn is_valid_for(self, service: ServiceId, endpoint: CapabilityHandle) -> bool {
        self.service as u16 == service as u16
            && self.endpoint == endpoint
            && self.endpoint.is_valid()
            && self.device == self.resources.device
    }

    pub fn permits(self, required: KernelCapability) -> bool {
        self.capabilities.contains(required)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingDecision {
    Reject,
    Accept,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrantError {
    ServiceMismatch,
    InvalidEndpoint,
    DeviceMismatch,
    MissingCapability,
    ResourceExceeded,
}

pub fn decide_binding(
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

pub fn validate_driver_grant(
    grant: DriverGrant,
    service: ServiceId,
    endpoint: CapabilityHandle,
    required: KernelCapability,
    allowed: ResourceGrant,
) -> Result<(), GrantError> {
    if grant.service as u16 != service as u16 {
        return Err(GrantError::ServiceMismatch);
    }
    if !endpoint.is_valid() || grant.endpoint != endpoint {
        return Err(GrantError::InvalidEndpoint);
    }
    if grant.device != grant.resources.device {
        return Err(GrantError::DeviceMismatch);
    }
    if grant.resources.device != allowed.device || !grant.resources.within(allowed) {
        return Err(GrantError::ResourceExceeded);
    }
    if !grant.permits(required) {
        return Err(GrantError::MissingCapability);
    }
    Ok(())
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
        let budget = ResourceGrant {
            device,
            mmio_bytes: 4096,
            io_bytes: 128,
            dma_bytes: 8192,
            interrupt_count: 4,
        };
        let requested = ResourceGrant {
            device,
            mmio_bytes: 2048,
            io_bytes: 64,
            dma_bytes: 4096,
            interrupt_count: 2,
        };
        assert!(requested.within(budget));
        let too_much = ResourceGrant {
            device,
            mmio_bytes: 8192,
            io_bytes: 64,
            dma_bytes: 4096,
            interrupt_count: 2,
        };
        assert!(!too_much.within(budget));
    }

    #[test]
    fn binding_is_fail_closed() {
        let matcher = DeviceMatch::exact(1, 2, DeviceClass::Network);
        let allowed = ResourceGrant {
            device: DeviceId(9),
            mmio_bytes: 4096,
            io_bytes: 64,
            dma_bytes: 4096,
            interrupt_count: 2,
        };
        let requested = ResourceGrant {
            device: DeviceId(9),
            mmio_bytes: 8192,
            io_bytes: 64,
            dma_bytes: 4096,
            interrupt_count: 2,
        };
        assert_eq!(
            decide_binding(matcher, 1, 2, DeviceClass::Network, requested, allowed),
            BindingDecision::Reject
        );
    }

    #[test]
    fn driver_grant_binds_capability_endpoint_and_device() {
        let device = DeviceId(21);
        let endpoint = CapabilityHandle(77);
        let caps = CapabilitySet::EMPTY
            .with(KernelCapability::DeviceGrant)
            .with(KernelCapability::Dma);
        let resources = ResourceGrant {
            device,
            mmio_bytes: 1024,
            io_bytes: 32,
            dma_bytes: 2048,
            interrupt_count: 1,
        };
        let grant = DriverGrant::new(ServiceId::Driverd, endpoint, device, resources, caps);
        assert_eq!(
            validate_driver_grant(
                grant,
                ServiceId::Driverd,
                endpoint,
                KernelCapability::Dma,
                resources
            ),
            Ok(())
        );
        assert_eq!(
            validate_driver_grant(
                grant,
                ServiceId::Appd,
                endpoint,
                KernelCapability::Dma,
                resources
            ),
            Err(GrantError::ServiceMismatch)
        );
    }

    #[test]
    fn driver_grant_rejects_missing_capability_and_excess_resources() {
        let device = DeviceId(22);
        let endpoint = CapabilityHandle(88);
        let allowed = ResourceGrant {
            device,
            mmio_bytes: 1024,
            io_bytes: 32,
            dma_bytes: 1024,
            interrupt_count: 1,
        };
        let requested = ResourceGrant {
            device,
            mmio_bytes: 2048,
            io_bytes: 32,
            dma_bytes: 1024,
            interrupt_count: 1,
        };
        let grant = DriverGrant::new(
            ServiceId::Driverd,
            endpoint,
            device,
            requested,
            CapabilitySet::EMPTY,
        );
        assert_eq!(
            validate_driver_grant(
                grant,
                ServiceId::Driverd,
                endpoint,
                KernelCapability::Dma,
                allowed
            ),
            Err(GrantError::ResourceExceeded)
        );
        let limited = DriverGrant::new(
            ServiceId::Driverd,
            endpoint,
            device,
            allowed,
            CapabilitySet::EMPTY,
        );
        assert_eq!(
            validate_driver_grant(
                limited,
                ServiceId::Driverd,
                endpoint,
                KernelCapability::Dma,
                allowed
            ),
            Err(GrantError::MissingCapability)
        );
    }
}
