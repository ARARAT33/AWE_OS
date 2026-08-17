#![no_std]

//! Bounded init/service-manager contracts. The kernel exposes only the
//! primitives needed for isolation; policy and lifecycle stay in userspace.

pub const INIT_ABI_MAJOR: u16 = 1;
pub const INIT_ABI_MINOR: u16 = 0;
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
pub enum ServiceError { Full, InvalidSpec, InvalidTransition, CapabilityDenied }

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
