use crate::{DriverCommand, DriverId, DriverRegistry, DriverState};

pub const DEFAULT_MAX_RESTARTS: u16 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupervisorError {
    Registry,
    InvalidTransition,
    UnknownDriver,
    RestartBudgetExhausted,
}

/// Small, deterministic lifecycle supervisor. Driver implementations are
/// deliberately outside this crate's kernel-facing ABI; this object only
/// controls their state and fault isolation policy.
pub struct DriverSupervisor {
    registry: DriverRegistry,
    restart_counts: [(DriverId, u16); crate::MAX_REGISTERED_DRIVERS],
    restart_len: usize,
    max_restarts: u16,
}

impl Default for DriverSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverSupervisor {
    pub const fn new() -> Self {
        Self {
            registry: DriverRegistry::new(),
            restart_counts: [(DriverId(0), 0); crate::MAX_REGISTERED_DRIVERS],
            restart_len: 0,
            max_restarts: DEFAULT_MAX_RESTARTS,
        }
    }

    pub const fn with_restart_budget(max_restarts: u16) -> Self {
        Self {
            max_restarts,
            ..Self::new()
        }
    }

    pub const fn registry(&self) -> &DriverRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut DriverRegistry {
        &mut self.registry
    }

    fn restart_count(&self, id: DriverId) -> u16 {
        let mut i = 0;
        while i < self.restart_len {
            if self.restart_counts[i].0 == id {
                return self.restart_counts[i].1;
            }
            i += 1;
        }
        0
    }

    fn record_restart(&mut self, id: DriverId) -> Result<(), SupervisorError> {
        let mut i = 0;
        while i < self.restart_len {
            if self.restart_counts[i].0 == id {
                self.restart_counts[i].1 = self.restart_counts[i].1.saturating_add(1);
                return Ok(());
            }
            i += 1;
        }
        if self.restart_len == self.restart_counts.len() {
            return Err(SupervisorError::RestartBudgetExhausted);
        }
        self.restart_counts[self.restart_len] = (id, 1);
        self.restart_len += 1;
        Ok(())
    }

    pub fn restart_count_for(&self, id: DriverId) -> u16 {
        self.restart_count(id)
    }

    pub fn command(
        &mut self,
        id: DriverId,
        command: DriverCommand,
    ) -> Result<DriverState, SupervisorError> {
        let current = self
            .registry
            .find(id)
            .ok_or(SupervisorError::UnknownDriver)?
            .state;

        if matches!(command, DriverCommand::Reset)
            && matches!(current, DriverState::Running | DriverState::Failed)
            && self.restart_count(id) >= self.max_restarts
        {
            return Err(SupervisorError::RestartBudgetExhausted);
        }

        let next = match (current, command) {
            (DriverState::Discovered, DriverCommand::Probe) => DriverState::Starting,
            (DriverState::Starting, DriverCommand::Start) => DriverState::Running,
            (DriverState::Running, DriverCommand::Stop) => DriverState::Stopping,
            (DriverState::Stopping, DriverCommand::Stop) => DriverState::Discovered,
            (DriverState::Running, DriverCommand::Reset) => {
                self.record_restart(id)?;
                DriverState::Starting
            }
            (_, DriverCommand::Quarantine) => DriverState::Quarantined,
            (DriverState::Running, DriverCommand::HealthCheck) => DriverState::Running,
            (DriverState::Failed, DriverCommand::Reset) => {
                self.record_restart(id)?;
                DriverState::Starting
            }
            (DriverState::Starting, DriverCommand::Reset) => {
                if self.restart_count(id) >= self.max_restarts {
                    return Err(SupervisorError::RestartBudgetExhausted);
                }
                self.record_restart(id)?;
                DriverState::Starting
            }
            _ => return Err(SupervisorError::InvalidTransition),
        };
        self.registry
            .set_state(id, next)
            .map_err(|_| SupervisorError::Registry)?;
        Ok(next)
    }

    pub fn fault(&mut self, id: DriverId) -> Result<(), SupervisorError> {
        self.registry
            .set_state(id, DriverState::Failed)
            .map_err(|_| SupervisorError::UnknownDriver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DriverClass, DriverDescriptor, DriverTrust};

    fn register(supervisor: &mut DriverSupervisor, id: u64) {
        supervisor
            .registry_mut()
            .register(DriverDescriptor {
                id: DriverId(id),
                class: DriverClass::Virtio,
                abi_major: 1,
                abi_minor: 2,
                vendor: 0x1af4,
                device: 0x1001,
                state: DriverState::Discovered,
                trust: DriverTrust::Verified,
            })
            .unwrap();
    }

    #[test]
    fn restart_policy_is_bounded_and_recovery_is_explicit() {
        let mut supervisor = DriverSupervisor::with_restart_budget(2);
        let id = DriverId(7);
        register(&mut supervisor, id.0);

        assert_eq!(
            supervisor.command(id, DriverCommand::Probe),
            Ok(DriverState::Starting)
        );
        assert_eq!(
            supervisor.command(id, DriverCommand::Start),
            Ok(DriverState::Running)
        );
        assert_eq!(
            supervisor.command(id, DriverCommand::Reset),
            Ok(DriverState::Starting)
        );
        assert_eq!(supervisor.restart_count_for(id), 1);
        assert_eq!(
            supervisor.command(id, DriverCommand::Start),
            Ok(DriverState::Running)
        );
        assert_eq!(
            supervisor.command(id, DriverCommand::Reset),
            Ok(DriverState::Starting)
        );
        assert_eq!(supervisor.restart_count_for(id), 2);
        assert_eq!(
            supervisor.command(id, DriverCommand::Reset),
            Err(SupervisorError::RestartBudgetExhausted)
        );
    }

    #[test]
    fn fault_path_is_fail_closed_and_quarantine_is_available() {
        let mut supervisor = DriverSupervisor::new();
        let id = DriverId(8);
        register(&mut supervisor, id.0);
        assert_eq!(supervisor.fault(id), Ok(()));
        assert_eq!(
            supervisor.registry().find(id).unwrap().state,
            DriverState::Failed
        );
        assert_eq!(
            supervisor.command(id, DriverCommand::Quarantine),
            Ok(DriverState::Quarantined)
        );
    }
}
