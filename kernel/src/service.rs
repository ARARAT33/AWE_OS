#![no_std]

//! 60.5 system-service process model.
//!
//! A service is a user-space process with an explicit owner class,
//! capability contract, and bounded resource budget. CellKernel owns only the
//! process/scheduling primitives; service implementation remains outside the
//! kernel.

use crate::process::{ProcessId, ResourceBudget};
use crate::system_contract::{CapabilitySet, KernelCapability, ServiceId, ServiceContract, APPD_CONTRACT, ASAPPD_CONTRACT, AYUID_CONTRACT, DRIVERD_CONTRACT};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceClass {
    System = 0,
    Hardware = 1,
    Application = 2,
    Interface = 3,
    Compatibility = 4,
    Update = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceState {
    Declared = 0,
    Starting = 1,
    Running = 2,
    Stopping = 3,
    Failed = 4,
    Quarantined = 5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceDescriptor {
    pub service: ServiceId,
    pub process: ProcessId,
    pub class: ServiceClass,
    pub state: ServiceState,
    pub capabilities: CapabilitySet,
    pub budget: ResourceBudget,
}

impl ServiceDescriptor {
    pub const fn new(service: ServiceId, process: ProcessId, class: ServiceClass, capabilities: CapabilitySet, budget: ResourceBudget) -> Self {
        Self { service, process, class, state: ServiceState::Declared, capabilities, budget }
    }

    pub const fn start(self) -> Self { Self { state: ServiceState::Starting, ..self } }
    pub const fn running(self) -> Self { Self { state: ServiceState::Running, ..self } }
    pub const fn stop(self) -> Self { Self { state: ServiceState::Stopping, ..self } }
    pub const fn fail(self) -> Self { Self { state: ServiceState::Failed, ..self } }
    pub const fn quarantine(self) -> Self { Self { state: ServiceState::Quarantined, ..self } }

    pub const fn requires(self, required: CapabilitySet) -> bool {
        self.capabilities.bits() & required.bits() == required.bits()
    }

    pub const fn can_start(self, required: CapabilitySet) -> bool {
        matches!(self.state, ServiceState::Declared) && self.requires(required)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceModelError {
    MissingCapability,
    Duplicate,
    Full,
    NotFound,
    InvalidState,
}

/// Fixed-size service ownership table. This is kernel metadata only; service
/// implementations live in user space and are never linked into CellKernel.
pub struct ServiceRegistry<const N: usize> {
    entries: [Option<ServiceDescriptor>; N],
    len: usize,
}

impl<const N: usize> ServiceRegistry<N> {
    pub const fn new() -> Self {
        Self { entries: [None; N], len: 0 }
    }

    pub const fn len(&self) -> usize { self.len }

    pub fn register(&mut self, descriptor: ServiceDescriptor) -> Result<(), ServiceModelError> {
        if self.find(descriptor.service).is_some() {
            return Err(ServiceModelError::Duplicate);
        }
        if self.len == N {
            return Err(ServiceModelError::Full);
        }
        self.entries[self.len] = Some(descriptor);
        self.len += 1;
        Ok(())
    }

    pub fn find(&self, service: ServiceId) -> Option<&ServiceDescriptor> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = &self.entries[i] {
                if entry.service == service {
                    return Some(entry);
                }
            }
            i += 1;
        }
        None
    }

    pub fn update_state(&mut self, service: ServiceId, state: ServiceState) -> Result<(), ServiceModelError> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = &mut self.entries[i] {
                if entry.service == service {
                    entry.state = state;
                    return Ok(());
                }
            }
            i += 1;
        }
        Err(ServiceModelError::NotFound)
    }
}

/// Canonical service roster for the 60.5 architecture freeze.
pub const SERVICE_COUNT: usize = 7;
pub const SERVICE_IDS: [ServiceId; SERVICE_COUNT] = [
    ServiceId::Driverd,
    ServiceId::Appd,
    ServiceId::Asappd,
    ServiceId::Ayuid,
    ServiceId::Aweterminald,
    ServiceId::Awebusd,
    ServiceId::Aweupdated,
];

pub const SERVICE_CLASSES: [ServiceClass; SERVICE_COUNT] = [
    ServiceClass::Hardware,
    ServiceClass::Application,
    ServiceClass::Application,
    ServiceClass::Interface,
    ServiceClass::Interface,
    ServiceClass::System,
    ServiceClass::Update,
];

pub const fn required_contract(service: ServiceId) -> Option<ServiceContract> {
    match service {
        ServiceId::Driverd => Some(DRIVERD_CONTRACT),
        ServiceId::Appd => Some(APPD_CONTRACT),
        ServiceId::Asappd => Some(ASAPPD_CONTRACT),
        ServiceId::Ayuid => Some(AYUID_CONTRACT),
        _ => None,
    }
}

/// Conservative bootstrap budget used by services until the runtime resource
/// manager negotiates per-machine quotas.
pub const fn bootstrap_budget() -> ResourceBudget {
    ResourceBudget {
        cpu_ticks: 1_000_000,
        memory_bytes: 64 * 1024 * 1024,
        ipc_messages: 65_536,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_caps() -> CapabilitySet {
        CapabilitySet::EMPTY
            .with(KernelCapability::Ipc)
            .with(KernelCapability::Security)
    }

    #[test]
    fn service_lifecycle_is_explicit() {
        let descriptor = ServiceDescriptor::new(ServiceId::Appd, ProcessId(7), ServiceClass::Application, sample_caps(), bootstrap_budget());
        assert_eq!(descriptor.state, ServiceState::Declared);
        assert_eq!(descriptor.start().state, ServiceState::Starting);
        assert_eq!(descriptor.start().running().state, ServiceState::Running);
        assert_eq!(descriptor.running().stop().state, ServiceState::Stopping);
        assert_eq!(descriptor.running().fail().state, ServiceState::Failed);
        assert_eq!(descriptor.running().quarantine().state, ServiceState::Quarantined);
    }

    #[test]
    fn capability_boundary_is_fail_closed() {
        let descriptor = ServiceDescriptor::new(ServiceId::Driverd, ProcessId(9), ServiceClass::Hardware, sample_caps(), bootstrap_budget());
        assert!(descriptor.requires(CapabilitySet::EMPTY.with(KernelCapability::Ipc)));
        assert!(!descriptor.requires(CapabilitySet::EMPTY.with(KernelCapability::Dma)));
        assert!(!descriptor.can_start(CapabilitySet::EMPTY.with(KernelCapability::Dma)));
    }

    #[test]
    fn registry_is_bounded_and_unique() {
        let mut registry: ServiceRegistry<2> = ServiceRegistry::new();
        assert!(registry.register(ServiceDescriptor::new(ServiceId::Driverd, ProcessId(1), ServiceClass::Hardware, sample_caps(), bootstrap_budget())).is_ok());
        assert_eq!(registry.register(ServiceDescriptor::new(ServiceId::Driverd, ProcessId(2), ServiceClass::Hardware, sample_caps(), bootstrap_budget())), Err(ServiceModelError::Duplicate));
        assert!(registry.register(ServiceDescriptor::new(ServiceId::Appd, ProcessId(2), ServiceClass::Application, sample_caps(), bootstrap_budget())).is_ok());
        assert_eq!(registry.register(ServiceDescriptor::new(ServiceId::Asappd, ProcessId(3), ServiceClass::Application, sample_caps(), bootstrap_budget())), Err(ServiceModelError::Full));
    }

    #[test]
    fn canonical_roster_is_stable() {
        assert_eq!(SERVICE_IDS.len(), SERVICE_COUNT);
        assert_eq!(SERVICE_CLASSES.len(), SERVICE_COUNT);
        assert_eq!(required_contract(ServiceId::Driverd), Some(DRIVERD_CONTRACT));
        assert_eq!(required_contract(ServiceId::Appd), Some(APPD_CONTRACT));
        assert_eq!(required_contract(ServiceId::Asappd), Some(ASAPPD_CONTRACT));
        assert_eq!(required_contract(ServiceId::Ayuid), Some(AYUID_CONTRACT));
        assert!(required_contract(ServiceId::Aweterminald).is_none());
    }
}
