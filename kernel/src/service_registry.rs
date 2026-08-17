#![no_std]

//! Fixed-capacity system-service registry for the 60.5-61.0 boundary.
//! Registration is kernel-owned metadata only; service implementation stays in
//! the user-space service plane.

use crate::service::{ServiceDescriptor, ServiceState};
use crate::system_contract::ServiceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError {
    Full,
    Duplicate,
    NotFound,
}

pub struct ServiceRegistry<const N: usize> {
    entries: [Option<ServiceDescriptor>; N],
    len: usize,
}

impl<const N: usize> ServiceRegistry<N> {
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            len: 0,
        }
    }
    pub const fn len(&self) -> usize {
        self.len
    }
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub fn register(&mut self, descriptor: ServiceDescriptor) -> Result<(), RegistryError> {
        if self.find(descriptor.service).is_some() {
            return Err(RegistryError::Duplicate);
        }
        if self.is_full() {
            return Err(RegistryError::Full);
        }
        self.entries[self.len] = Some(descriptor);
        self.len += 1;
        Ok(())
    }

    pub fn find(&self, service: ServiceId) -> Option<&ServiceDescriptor> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = &self.entries[i]
                && entry.service as u16 == service as u16
            {
                return Some(entry);
            }
            i += 1;
        }
        None
    }

    pub fn update_state(
        &mut self,
        service: ServiceId,
        state: ServiceState,
    ) -> Result<(), RegistryError> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = &mut self.entries[i]
                && entry.service as u16 == service as u16
            {
                entry.state = state;
                return Ok(());
            }
            i += 1;
        }
        Err(RegistryError::NotFound)
    }
}

impl<const N: usize> Default for ServiceRegistry<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Canonical seven-service namespace defined by the 60.2 contract.
pub const CANONICAL_SERVICE_COUNT: usize = 7;
pub const CANONICAL_SERVICES: [ServiceId; CANONICAL_SERVICE_COUNT] = [
    ServiceId::Driverd,
    ServiceId::Appd,
    ServiceId::Asappd,
    ServiceId::Ayuid,
    ServiceId::Aweterminald,
    ServiceId::Awebusd,
    ServiceId::Aweupdated,
];

pub const fn is_canonical_service(service: ServiceId) -> bool {
    match service {
        ServiceId::Driverd
        | ServiceId::Appd
        | ServiceId::Asappd
        | ServiceId::Ayuid
        | ServiceId::Aweterminald
        | ServiceId::Awebusd
        | ServiceId::Aweupdated => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{ProcessId, ResourceBudget};
    use crate::service::ServiceClass;
    use crate::system_contract::{CapabilitySet, KernelCapability};

    fn descriptor(id: ServiceId, process: u64) -> ServiceDescriptor {
        ServiceDescriptor::new(
            id,
            ProcessId(process),
            ServiceClass::System,
            CapabilitySet::EMPTY.with(KernelCapability::Ipc),
            ResourceBudget {
                cpu_ticks: 10,
                memory_bytes: 1024,
                ipc_messages: 4,
            },
        )
    }

    #[test]
    fn registry_is_bounded_and_deduplicated() {
        let mut registry: ServiceRegistry<2> = ServiceRegistry::new();
        assert_eq!(registry.register(descriptor(ServiceId::Appd, 1)), Ok(()));
        assert_eq!(
            registry.register(descriptor(ServiceId::Appd, 2)),
            Err(RegistryError::Duplicate)
        );
        assert_eq!(registry.register(descriptor(ServiceId::Ayuid, 3)), Ok(()));
        assert_eq!(
            registry.register(descriptor(ServiceId::Driverd, 4)),
            Err(RegistryError::Full)
        );
    }

    #[test]
    fn canonical_roster_is_complete() {
        assert_eq!(CANONICAL_SERVICES.len(), CANONICAL_SERVICE_COUNT);
        assert!(is_canonical_service(ServiceId::Driverd));
        assert!(is_canonical_service(ServiceId::Aweupdated));
    }

    #[test]
    fn lifecycle_state_can_be_published_through_registry() {
        let mut registry: ServiceRegistry<2> = ServiceRegistry::new();
        registry.register(descriptor(ServiceId::Appd, 1)).unwrap();
        assert_eq!(
            registry.update_state(ServiceId::Appd, ServiceState::Running),
            Ok(())
        );
        assert_eq!(
            registry.find(ServiceId::Appd).unwrap().state,
            ServiceState::Running
        );
    }
}
