#![no_std]

//! Bounded init/service-manager contracts. The kernel exposes only the
//! primitives needed for isolation; policy and lifecycle stay in userspace.

pub const INIT_ABI_MAJOR: u16 = 1;
pub const INIT_ABI_MINOR: u16 = 2;
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
pub enum ServiceError { Full, InvalidSpec, InvalidTransition, CapabilityDenied, Duplicate, Quarantined }

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

pub const fn restart_allowed(policy: RestartPolicy, state: ServiceState) -> bool {
    match (policy, state) {
        (RestartPolicy::Never, _) => false,
        (RestartPolicy::OnFailure, ServiceState::Failed) => true,
        (RestartPolicy::Always, ServiceState::Failed | ServiceState::Stopped) => true,
        _ => false,
    }
}

/// Fixed-capacity userspace service table. No allocation is required and
/// duplicate registration is rejected before lifecycle changes are allowed.
pub struct ServiceTable {
    specs: [Option<ServiceSpec>; MAX_SERVICES],
    states: [ServiceState; MAX_SERVICES],
    count: usize,
}

impl ServiceTable {
    pub const fn new() -> Self {
        Self { specs: [None; MAX_SERVICES], states: [ServiceState::Stopped; MAX_SERVICES], count: 0 }
    }

    pub const fn len(&self) -> usize { self.count }
    pub const fn is_empty(&self) -> bool { self.count == 0 }

    pub fn register(&mut self, spec: ServiceSpec) -> Result<(), ServiceError> {
        validate_spec(spec)?;
        let index = spec.id.0 as usize;
        if self.specs[index].is_some() { return Err(ServiceError::Duplicate); }
        if self.count == MAX_SERVICES { return Err(ServiceError::Full); }
        self.specs[index] = Some(spec);
        self.states[index] = ServiceState::Declared;
        self.count += 1;
        Ok(())
    }

    pub const fn state(&self, id: ServiceId) -> Option<ServiceState> {
        if id.0 as usize >= MAX_SERVICES { return None; }
        if self.specs[id.0 as usize].is_some() { Some(self.states[id.0 as usize]) } else { None }
    }

    pub fn set_state(&mut self, id: ServiceId, next: ServiceState) -> Result<(), ServiceError> {
        let index = id.0 as usize;
        if index >= MAX_SERVICES || self.specs[index].is_none() { return Err(ServiceError::InvalidSpec); }
        if self.states[index] == ServiceState::Quarantined { return Err(ServiceError::Quarantined); }
        if !transition(self.states[index], next) { return Err(ServiceError::InvalidTransition); }
        self.states[index] = next;
        Ok(())
    }

    pub fn restart(&mut self, id: ServiceId) -> Result<(), ServiceError> {
        let index = id.0 as usize;
        if index >= MAX_SERVICES || self.specs[index].is_none() { return Err(ServiceError::InvalidSpec); }
        let spec = self.specs[index].unwrap();
        let state = self.states[index];
        if state == ServiceState::Quarantined { return Err(ServiceError::Quarantined); }
        if !restart_allowed(spec.restart, state) { return Err(ServiceError::InvalidTransition); }
        self.states[index] = ServiceState::Starting;
        Ok(())
    }

    pub const fn spec(&self, id: ServiceId) -> Option<ServiceSpec> {
        if id.0 as usize >= MAX_SERVICES { None } else { self.specs[id.0 as usize] }
    }
}

impl Default for ServiceTable { fn default() -> Self { Self::new() } }

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
        assert_eq!(table.len(), 1);
        table.set_state(ServiceId(1), ServiceState::Starting).unwrap();
        table.set_state(ServiceId(1), ServiceState::Running).unwrap();
        assert_eq!(table.spec(ServiceId(1)), Some(SPEC));
    }
    #[test]
    fn failed_service_restarts_only_when_policy_allows() {
        let mut table = ServiceTable::new();
        table.register(SPEC).unwrap();
        table.set_state(ServiceId(1), ServiceState::Starting).unwrap();
        table.set_state(ServiceId(1), ServiceState::Failed).unwrap();
        table.restart(ServiceId(1)).unwrap();
        assert_eq!(table.state(ServiceId(1)), Some(ServiceState::Starting));
    }
    #[test]
    fn quarantined_service_cannot_restart() {
        let mut table = ServiceTable::new();
        table.register(SPEC).unwrap();
        table.set_state(ServiceId(1), ServiceState::Starting).unwrap();
        table.set_state(ServiceId(1), ServiceState::Failed).unwrap();
        table.set_state(ServiceId(1), ServiceState::Quarantined).unwrap();
        assert_eq!(table.restart(ServiceId(1)), Err(ServiceError::Quarantined));
    }
    #[test]
    fn duplicate_registration_is_rejected() {
        let mut table = ServiceTable::new();
        table.register(SPEC).unwrap();
        assert_eq!(table.register(SPEC), Err(ServiceError::Duplicate));
    }
}
