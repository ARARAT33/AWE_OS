use crate::{DriverCommand, DriverId, DriverRegistry, DriverState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupervisorError {
    Registry,
    InvalidTransition,
    UnknownDriver,
}

/// Small, deterministic lifecycle supervisor. Driver implementations are
/// deliberately outside this crate's kernel-facing ABI; this object only
/// controls their state and fault isolation policy.
pub struct DriverSupervisor {
    registry: DriverRegistry,
}

impl DriverSupervisor {
    pub const fn new() -> Self { Self { registry: DriverRegistry::new() } }
    pub const fn registry(&self) -> &DriverRegistry { &self.registry }
    pub fn registry_mut(&mut self) -> &mut DriverRegistry { &mut self.registry }

    pub fn command(&mut self, id: DriverId, command: DriverCommand) -> Result<DriverState, SupervisorError> {
        let current = self.registry.find(id).ok_or(SupervisorError::UnknownDriver)?.state;
        let next = match (current, command) {
            (DriverState::Discovered, DriverCommand::Probe) => DriverState::Starting,
            (DriverState::Starting, DriverCommand::Start) => DriverState::Running,
            (DriverState::Running, DriverCommand::Stop) => DriverState::Stopping,
            (DriverState::Stopping, DriverCommand::Stop) => DriverState::Discovered,
            (DriverState::Running, DriverCommand::Reset) => DriverState::Starting,
            (_, DriverCommand::Quarantine) => DriverState::Quarantined,
            (DriverState::Running, DriverCommand::HealthCheck) => DriverState::Running,
            (DriverState::Failed, DriverCommand::Reset) => DriverState::Starting,
            _ => return Err(SupervisorError::InvalidTransition),
        };
        self.registry.set_state(id, next).map_err(|_| SupervisorError::Registry)?;
        Ok(next)
    }

    pub fn fault(&mut self, id: DriverId) -> Result<(), SupervisorError> {
        self.registry.set_state(id, DriverState::Failed).map_err(|_| SupervisorError::UnknownDriver)
    }
}
