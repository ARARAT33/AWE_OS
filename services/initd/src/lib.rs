#![no_std]

//! Bounded init/service-manager contracts. The kernel exposes only the
//! primitives needed for isolation; policy and lifecycle stay in userspace.

pub const INIT_ABI_MAJOR: u16 = 1;
pub const INIT_ABI_MINOR: u16 = 1;
pub const MAX_SERVICES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceState { Declared, Starting, Running, Stopping, Stopped, Failed, Quarantined }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestartPolicy { Never, OnFailure, Always }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceSpec {
    pub id: ServiceId,
    pub restart: RestartPolicy,
    pub capability_mask: u64,
    pub memory_limit_pages: u32,
    pub cpu_budget_ticks: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceError { Full, InvalidSpec, InvalidTransition, CapabilityDenied, Duplicate }

pub const fn validate_spec(spec: ServiceSpec) -> Result<(), ServiceError> {
    if spec.id.0 as usize >= MAX_SERVICES || spec.memory_limit_pages == 0 || spec.cpu_budget_ticks == 0 {
        return Err(ServiceError::InvalidSpec);
    }
    Ok(())
}

pub const fn transition(from: ServiceState, to: ServiceState) -> bool {
    matches!((from, to),
        (ServiceState::Declared, ServiceState::Starting) |
        (ServiceState::Starting, ServiceState::Running) |
        (ServiceState::Starting, ServiceState::Failed) |
        (ServiceState::Running, ServiceState::Stopping) |
        (ServiceState::Running, ServiceState::Failed) |
        (ServiceState::Stopping, ServiceState::Stopped) |
        (ServiceState::Failed, ServiceState::Starting) |
        (ServiceState::Failed, ServiceState::Quarantined) |
        (ServiceState::Stopped, ServiceState::Starting))
}

/// Fixed-capacity userspace service table. No allocation is required and
/// duplicate registration is rejected before lifecycle changes are allowed.
pub struct ServiceTable {
    specs: [Option<ServiceSpec>; MAX_SERVICES],
    states: [ServiceState; MAX_SERVICES],
}

impl ServiceTable {
    pub const fn new() -> Self {
        Self { specs: [None; MAX_SERVICES], states: [ServiceState::Stopped; MAX_SERVICES] }
    }

    pub fn register(&mut self, spec: ServiceSpec) -> Result<(), ServiceError> {
        validate_spec(spec)?;
        let index = spec.id.0 as usize;
        if self.specs[index].is_some() { return Err(ServiceError::Duplicate); }
        self.specs[index] = Some(spec);
        self.states[index] = ServiceState::Declared;
        Ok(())
    }

    pub const fn state(&self, id: ServiceId) -> Option<ServiceState> {
        if id.0 as usize >= MAX_SERVICES { return None; }
        if self.specs[id.0 as usize].is_some() { Some(self.states[id.0 as usize]) } else { None }
    }

    pub fn set_state(&mut self, id: ServiceId, next: ServiceState) -> Result<(), ServiceError> {
        let index = id.0 as usize;
        if index >= MAX_SERVICES || self.specs[index].is_none() { return Err(ServiceError::InvalidSpec); }
        if !transition(self.states[index], next) { return Err(ServiceError::InvalidTransition); }
        self.states[index] = next;
        Ok(())
    }

    pub const fn spec(&self, id: ServiceId) -> Option<ServiceSpec> {
        if id.0 as usize >= MAX_SERVICES { None } else { self.specs[id.0 as usize] }
    }
}

impl Default for ServiceTable {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: ServiceSpec = ServiceSpec { id: ServiceId(1), restart: RestartPolicy::OnFailure, capability_mask: 3, memory_limit_pages: 4, cpu_budget_ticks: 10 };

    #[test]
    fn rejects_unbounded_service_specs() {
        let spec = ServiceSpec { id: ServiceId(MAX_SERVICES as u16), restart: RestartPolicy::Always, capability_mask: 0, memory_limit_pages: 1, cpu_budget_ticks: 1 };
        assert_eq!(validate_spec(spec), Err(ServiceError::InvalidSpec));
    }

    #[test]
    fn lifecycle_is_fail_closed() {
        assert!(transition(ServiceState::Starting, ServiceState::Running));
        assert!(!transition(ServiceState::Stopped, ServiceState::Running));
    }

    #[test]
    fn table_registers_and_runs_a_service() {
        let mut table = ServiceTable::new();
        table.register(SPEC).unwrap();
        assert_eq!(table.state(ServiceId(1)), Some(ServiceState::Declared));
        table.set_state(ServiceId(1), ServiceState::Starting).unwrap();
        table.set_state(ServiceId(1), ServiceState::Running).unwrap();
        assert_eq!(table.spec(ServiceId(1)), Some(SPEC));
    }

    #[test]
    fn duplicate_registration_is_rejected() {
        let mut table = ServiceTable::new();
        table.register(SPEC).unwrap();
        assert_eq!(table.register(SPEC), Err(ServiceError::Duplicate));
    }
}
