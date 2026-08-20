#![no_std]

use super::linux_driver_health::{DriverHealthMonitor, HealthError, HealthState};
use super::linux_multi_instance::{MultiInstanceError, MultiInstanceManager};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecoveryError {
    Health(HealthError),
    Instance(MultiInstanceError),
    InvalidIndex,
    RecoveryLimit,
}

/// Coordinates health state, isolation and bounded recovery for one driver instance.
pub struct FaultRecovery<const N: usize> {
    pub health: DriverHealthMonitor<N>,
    pub attempts: [u8; N],
    pub max_attempts: u8,
}

impl<const N: usize> FaultRecovery<N> {
    pub const fn new(max_attempts: u8) -> Self {
        Self {
            health: DriverHealthMonitor::new(),
            attempts: [0; N],
            max_attempts,
        }
    }

    pub fn attach(&mut self) -> Result<usize, RecoveryError> {
        self.health.attach().map_err(RecoveryError::Health)
    }

    pub fn probe_result(&mut self, index: usize, success: bool) -> Result<(), RecoveryError> {
        self.health
            .report_probe(index, success)
            .map_err(RecoveryError::Health)
    }

    pub fn recover<const M: usize>(
        &mut self,
        index: usize,
        manager: &mut MultiInstanceManager<M>,
    ) -> Result<(), RecoveryError> {
        if index >= self.health.count {
            return Err(RecoveryError::InvalidIndex);
        }
        if self.attempts[index] >= self.max_attempts {
            return Err(RecoveryError::RecoveryLimit);
        }
        if self.health.entries[index].state == HealthState::Quarantined {
            return Err(RecoveryError::Health(HealthError::Quarantined));
        }
        self.attempts[index] = self.attempts[index].saturating_add(1);
        manager
            .rollback_instance(index)
            .map_err(RecoveryError::Instance)?;
        manager.probe(index).map_err(RecoveryError::Instance)?;
        manager.init(index).map_err(RecoveryError::Instance)?;
        manager.start(index).map_err(RecoveryError::Instance)?;
        self.health
            .request_recovery(index)
            .map_err(RecoveryError::Health)?;
        Ok(())
    }

    pub fn quarantine<const M: usize>(
        &mut self,
        index: usize,
        manager: &mut MultiInstanceManager<M>,
    ) -> Result<(), RecoveryError> {
        if index >= self.health.count {
            return Err(RecoveryError::InvalidIndex);
        }
        if self.health.entries[index].state != HealthState::Quarantined {
            self.health.entries[index].state = HealthState::Quarantined;
        }
        manager
            .rollback_instance(index)
            .map_err(RecoveryError::Instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_recovery_restarts_instance() {
        let mut r = FaultRecovery::<1>::new(2);
        let i = r.attach().unwrap();
        let mut m = MultiInstanceManager::<1>::new();
        m.add(100).unwrap();
        m.activate_all().unwrap();
        r.probe_result(i, false).unwrap();
        r.recover(i, &mut m).unwrap();
        assert_eq!(r.health.entries[i].state, HealthState::Healthy);
        assert_eq!(m.instance(0).unwrap().active, true);
        assert_eq!(r.attempts[i], 1);
    }

    #[test]
    fn recovery_limit_is_enforced() {
        let mut r = FaultRecovery::<1>::new(1);
        let i = r.attach().unwrap();
        let mut m = MultiInstanceManager::<1>::new();
        m.add(7).unwrap();
        m.activate_all().unwrap();
        r.probe_result(i, false).unwrap();
        r.recover(i, &mut m).unwrap();
        assert_eq!(r.recover(i, &mut m), Err(RecoveryError::RecoveryLimit));
    }
}
