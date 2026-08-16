#![no_std]

use super::linux_driver_health::{DriverHealthMonitor, HealthError, HealthState};
use super::linux_driver_registry::{DriverRecord, DriverRegistry, RegistryError};
use super::linux_driver_ops::{DriverOp, DriverState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SupervisorError {
    Registry(RegistryError),
    Health(HealthError),
    RecoveryUnavailable,
}

/// Coordinates registry state and health state for bounded, allocation-free driver recovery.
pub struct DriverSupervisor<const N: usize> {
    pub registry: DriverRegistry<N>,
    pub health: DriverHealthMonitor<N>,
}

impl<const N: usize> DriverSupervisor<N> {
    pub const fn new() -> Self {
        Self { registry: DriverRegistry::new(), health: DriverHealthMonitor::new() }
    }

    pub fn register(&mut self, id: u64, class: u16, vendor: u16, device: u16) -> Result<usize, SupervisorError> {
        let index = self.registry.register(id, class, vendor, device).map_err(SupervisorError::Registry)?;
        if self.health.attach().is_err() {
            let _ = self.registry.unregister(id);
            return Err(SupervisorError::Health(HealthError::Full));
        }
        Ok(index)
    }

    pub fn apply(&mut self, id: u64, op: DriverOp, success: bool) -> Result<DriverState, SupervisorError> {
        let state = self.registry.apply(id, op, success);
        let health_index = self.registry_index(id).map_err(SupervisorError::Registry)?;
        self.health.report_probe(health_index, success).map_err(SupervisorError::Health)?;
        state.map_err(SupervisorError::Registry)
    }

    /// Retries a faulted driver from its last valid lifecycle checkpoint.
    pub fn recover(&mut self, id: u64) -> Result<DriverState, SupervisorError> {
        let index = self.registry_index(id).map_err(SupervisorError::Registry)?;
        if self.health.entries[index].state != HealthState::Faulted {
            return Err(SupervisorError::RecoveryUnavailable);
        }
        self.health.request_recovery(index).map_err(SupervisorError::Health)?;
        let record = self.registry.get(id).map_err(SupervisorError::Registry)?;
        match record.lifecycle {
            DriverState::New => {
                self.registry.apply(id, DriverOp::Probe, true).map_err(SupervisorError::Registry)?;
                self.registry.apply(id, DriverOp::Init, true).map_err(SupervisorError::Registry)?;
                self.registry.apply(id, DriverOp::Start, true).map_err(SupervisorError::Registry)
            }
            DriverState::Probed => {
                self.registry.apply(id, DriverOp::Init, true).map_err(SupervisorError::Registry)?;
                self.registry.apply(id, DriverOp::Start, true).map_err(SupervisorError::Registry)
            }
            DriverState::Initialized => self.registry.apply(id, DriverOp::Start, true).map_err(SupervisorError::Registry),
            DriverState::Running => Ok(DriverState::Running),
            DriverState::Stopped | DriverState::Removed => Err(SupervisorError::RecoveryUnavailable),
        }
    }

    pub fn get(&self, id: u64) -> Result<DriverRecord, SupervisorError> {
        self.registry.get(id).map_err(SupervisorError::Registry)
    }

    pub fn health(&self, id: u64) -> Result<HealthState, SupervisorError> {
        let index = self.registry_index(id).map_err(SupervisorError::Registry)?;
        Ok(self.health.entries[index].state)
    }

    fn registry_index(&self, id: u64) -> Result<usize, RegistryError> {
        let mut i = 0;
        while i < self.registry.len() {
            if self.registry.get_at(i)?.id == id { return Ok(i); }
            i += 1;
        }
        Err(RegistryError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supervisor_recovers_a_failed_start() {
        let mut s = DriverSupervisor::<2>::new();
        s.register(100, 1, 2, 3).unwrap();
        s.apply(100, DriverOp::Probe, true).unwrap();
        s.apply(100, DriverOp::Init, true).unwrap();
        assert_eq!(s.apply(100, DriverOp::Start, false), Err(SupervisorError::Registry(RegistryError::Lifecycle(super::super::linux_driver_ops::DriverOpError::StartFailed))));
        assert_eq!(s.health(100).unwrap(), HealthState::Faulted);
        assert_eq!(s.recover(100).unwrap(), DriverState::Running);
        assert_eq!(s.health(100).unwrap(), HealthState::Healthy);
    }

    #[test]
    fn third_failure_quarantines_and_blocks_recovery() {
        let mut s = DriverSupervisor::<1>::new();
        s.register(7, 1, 2, 3).unwrap();
        s.apply(7, DriverOp::Probe, false).unwrap_err();
        s.apply(7, DriverOp::Probe, false).unwrap_err();
        s.apply(7, DriverOp::Probe, false).unwrap_err();
        assert_eq!(s.health(7).unwrap(), HealthState::Quarantined);
        assert_eq!(s.recover(7), Err(SupervisorError::RecoveryUnavailable));
    }
}
