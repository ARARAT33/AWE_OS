#![no_std]

//! 60.5 system-service process model.
//!
//! A service is a user-space process with an explicit owner class,
//! capability contract, and bounded resource budget. CellKernel owns only the
//! process/scheduling primitives; service implementation remains outside the
//! kernel.

use crate::process::{ProcessId, ResourceBudget};
use crate::system_contract::{
    CapabilitySet, ServiceContract, ServiceId, APPD_CONTRACT, ASAPPD_CONTRACT,
    AWEBUSD_CONTRACT, AWEUPDATED_CONTRACT, AWETERMINALD_CONTRACT, AYUID_CONTRACT,
    DRIVERD_CONTRACT,
};

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
        Self {
            state: ServiceState::Starting,
            ..self
        }
    }

    pub const fn running(self) -> Self {
        Self {
            state: ServiceState::Running,
            ..self
        }
    }

    pub const fn stop(self) -> Self {
        Self {
            state: ServiceState::Stopping,
            ..self
        }
    }

    pub const fn fail(self) -> Self {
        Self {
            state: ServiceState::Failed,
            ..self
        }
    }

    pub const fn quarantine(self) -> Self {
        Self {
            state: ServiceState::Quarantined,
            ..self
        }
    }

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
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn find(&self, service: ServiceId) -> Option<ServiceDescriptor> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = self.entries[i] {
                if entry.service == service {
                    return Some(entry);
                }
            }
            i += 1;
        }
        None
    }
}

pub const fn canonical_contracts() -> [ServiceContract; 7] {
    [
        DRIVERD_CONTRACT,
        APPD_CONTRACT,
        ASAPPD_CONTRACT,
        AYUID_CONTRACT,
        AWETERMINALD_CONTRACT,
        AWEBUSD_CONTRACT,
        AWEUPDATED_CONTRACT,
    ]
}
