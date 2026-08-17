#![no_std]

//! 60.5 system-service process model.
//!
//! A service is a user-space process with an explicit owner class,
//! capability contract, and bounded resource budget. CellKernel owns only the
//! process/scheduling primitives; service implementation remains outside the
//! kernel.

use crate::process::{ProcessId, ResourceBudget};
use crate::system_contract::{CapabilitySet, ServiceId};

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
    pub const fn new(
        service: ServiceId,
        process: ProcessId,
        class: ServiceClass,
        capabilities: CapabilitySet,
        budget: ResourceBudget,
    ) -> Self {
        Self {
            service,
            process,
            class,
            state: ServiceState::Declared,
            capabilities,
            budget,
        }
    }

    pub const fn start(self) -> Self {
        Self { state: ServiceState::Starting, ..self }
    }

    pub const fn running(self) -> Self {
        Self { state: ServiceState::Running, ..self }
    }

    pub const fn stop(self) -> Self {
        Self { state: ServiceState::Stopping, ..self }
    }

    pub const fn fail(self) -> Self {
        Self { state: ServiceState::Failed, ..self }
    }

    pub const fn quarantine(self) -> Self {
        Self { state: ServiceState::Quarantined, ..self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceModelError {
    MissingCapability,
    InvalidState,
}

impl ServiceDescriptor {
    pub const fn requires(self, required: CapabilitySet) -> bool {
        self.capabilities.bits() & required.bits() == required.bits()
    }

    pub const fn can_start(self, required: CapabilitySet) -> bool {
        matches!(self.state, ServiceState::Declared) && self.requires(required)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessId;
    use crate::system_contract::{KernelCapability, CapabilitySet};

    #[test]
    fn service_lifecycle_is_explicit() {
        let caps = CapabilitySet::EMPTY.with(KernelCapability::Ipc);
        let budget = ResourceBudget { cpu_ticks: 100, memory_bytes: 4096, ipc_messages: 8 };
        let descriptor = ServiceDescriptor::new(
            ServiceId::Appd,
            ProcessId(7),
            ServiceClass::Application,
            caps,
            budget,
        );
        assert_eq!(descriptor.state, ServiceState::Declared);
        assert_eq!(descriptor.start().state, ServiceState::Starting);
        assert_eq!(descriptor.start().running().state, ServiceState::Running);
        assert_eq!(descriptor.running().stop().state, ServiceState::Stopping);
        assert_eq!(descriptor.running().fail().state, ServiceState::Failed);
        assert_eq!(descriptor.running().quarantine().state, ServiceState::Quarantined);
    }

    #[test]
    fn capability_boundary_is_fail_closed() {
        let caps = CapabilitySet::EMPTY.with(KernelCapability::Ipc);
        let descriptor = ServiceDescriptor::new(
            ServiceId::Driverd,
            ProcessId(9),
            ServiceClass::Hardware,
            caps,
            ResourceBudget { cpu_ticks: 1, memory_bytes: 1, ipc_messages: 1 },
        );
        assert!(descriptor.requires(CapabilitySet::EMPTY.with(KernelCapability::Ipc)));
        assert!(!descriptor.requires(CapabilitySet::EMPTY.with(KernelCapability::Dma)));
        assert!(!descriptor.can_start(CapabilitySet::EMPTY.with(KernelCapability::Dma)));
    }
}
