#![no_std]

use super::linux_multi_instance::MultiInstanceManager;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HealthState {
    Unknown,
    Healthy,
    Degraded,
    Faulted,
    Quarantined,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HealthError {
    Full,
    InvalidIndex,
    Quarantined,
    RecoveryLimit,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverHealth {
    pub state: HealthState,
    pub probes: u32,
    pub failures: u32,
    pub recoveries: u32,
    pub consecutive_failures: u16,
}
impl DriverHealth {
    pub const fn new() -> Self {
        Self {
            state: HealthState::Unknown,
            probes: 0,
            failures: 0,
            recoveries: 0,
            consecutive_failures: 0,
        }
    }
    pub fn healthy(&mut self) {
        self.probes = self.probes.saturating_add(1);
        self.consecutive_failures = 0;
        self.state = HealthState::Healthy
    }
    pub fn fault(&mut self) {
        self.failures = self.failures.saturating_add(1);
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.state = if self.consecutive_failures >= 3 {
            HealthState::Quarantined
        } else {
            HealthState::Faulted
        }
    }
    pub fn recovered(&mut self) {
        self.recoveries = self.recoveries.saturating_add(1);
        self.consecutive_failures = 0;
        self.state = HealthState::Healthy
    }
}

impl Default for DriverHealth {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverHealthMonitor<const N: usize> {
    pub entries: [DriverHealth; N],
    pub count: usize,
}
impl<const N: usize> DriverHealthMonitor<N> {
    pub const fn new() -> Self {
        Self {
            entries: [DriverHealth::new(); N],
            count: 0,
        }
    }
    pub fn attach(&mut self) -> Result<usize, HealthError> {
        if self.count == N {
            return Err(HealthError::Full);
        }
        let i = self.count;
        self.entries[i] = DriverHealth::new();
        self.count += 1;
        Ok(i)
    }
    pub fn report_probe(&mut self, index: usize, success: bool) -> Result<(), HealthError> {
        let h = self
            .entries
            .get_mut(index)
            .ok_or(HealthError::InvalidIndex)?;
        if h.state == HealthState::Quarantined {
            return Err(HealthError::Quarantined);
        }
        if success {
            h.healthy()
        } else {
            h.fault()
        }
        Ok(())
    }
    pub fn record_success(&mut self, index: usize) -> Result<(), HealthError> {
        self.report_probe(index, true)
    }
    pub fn record_failure(&mut self, index: usize) -> Result<(), HealthError> {
        self.report_probe(index, false)
    }
    pub fn request_recovery(&mut self, index: usize) -> Result<(), HealthError> {
        let h = self
            .entries
            .get_mut(index)
            .ok_or(HealthError::InvalidIndex)?;
        if h.state == HealthState::Quarantined {
            return Err(HealthError::Quarantined);
        }
        if h.state != HealthState::Faulted && h.state != HealthState::Degraded {
            return Err(HealthError::RecoveryLimit);
        }
        h.recovered();
        Ok(())
    }
    pub fn is_available(&self, index: usize) -> Result<bool, HealthError> {
        let h = self.entries.get(index).ok_or(HealthError::InvalidIndex)?;
        Ok(h.state != HealthState::Quarantined)
    }
    pub fn isolate<const M: usize>(
        &self,
        manager: &mut MultiInstanceManager<M>,
        instance: usize,
    ) -> Result<(), HealthError> {
        if !self.is_available(instance)? {
            return Err(HealthError::Quarantined);
        }
        manager
            .rollback_instance(instance)
            .map_err(|_| HealthError::InvalidIndex)
    }
}

impl<const N: usize> Default for DriverHealthMonitor<N> {
    fn default() -> Self {
        Self::new()
    }
}
