#![no_std]
use super::linux_dependency_multi_instance::DependencyMultiError;
use super::linux_driver_health::{DriverHealthMonitor, HealthError, HealthState};
use super::linux_driver_ops::{DriverLifecycle, DriverOp, DriverOpError, DriverState};
use super::linux_fault_impact::{FaultImpact, FaultImpactError};
use super::linux_fault_recovery::{FaultRecovery, RecoveryError};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecoveryPipelineError {
    Health(HealthError),
    Impact(FaultImpactError),
    Recovery(RecoveryError),
    Dependency(DependencyMultiError),
    Lifecycle(DriverOpError),
    Capacity,
    InconsistentState,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RecoveryReport<const N: usize> {
    pub failed_driver: u64,
    pub affected: [u64; N],
    pub affected_count: usize,
    pub recovered: bool,
    pub quarantined: bool,
}
pub struct RecoveryPipeline<const N: usize> {
    pub health: DriverHealthMonitor<N>,
    pub impact: FaultImpact<N>,
    pub recovery: FaultRecovery<N>,
}
impl<const N: usize> RecoveryPipeline<N> {
    pub const fn new() -> Self {
        Self {
            health: DriverHealthMonitor::new(),
            impact: FaultImpact::new(),
            recovery: FaultRecovery::new(3),
        }
    }
    fn health_index(driver: u64) -> usize {
        (driver as usize) % N
    }
    pub fn record_success(&mut self, driver: u64) -> Result<(), RecoveryPipelineError> {
        let i = Self::health_index(driver);
        if i >= self.health.count {
            self.health
                .attach()
                .map_err(RecoveryPipelineError::Health)?;
        }
        self.health
            .record_success(i)
            .map_err(RecoveryPipelineError::Health)
    }
    pub fn record_failure(&mut self, driver: u64) -> Result<HealthState, RecoveryPipelineError> {
        let i = Self::health_index(driver);
        if i >= self.health.count {
            self.health
                .attach()
                .map_err(RecoveryPipelineError::Health)?;
        }
        self.health
            .record_failure(i)
            .map_err(RecoveryPipelineError::Health)?;
        Ok(self.health.entries[i].state)
    }
    pub fn analyze(
        &mut self,
        failed: u64,
        edges: &[(u64, u64)],
    ) -> Result<usize, RecoveryPipelineError> {
        self.impact
            .compute_pairs(failed, edges)
            .map_err(RecoveryPipelineError::Impact)
    }
    pub fn recover_instance(
        &mut self,
        driver: &mut DriverLifecycle,
    ) -> Result<(), RecoveryPipelineError> {
        if driver.state == DriverState::Running {
            driver
                .apply(DriverOp::Stop, true)
                .map_err(RecoveryPipelineError::Lifecycle)?
        }
        if driver.state == DriverState::Stopped {
            driver
                .apply(DriverOp::Remove, true)
                .map_err(RecoveryPipelineError::Lifecycle)?
        }
        if driver.state == DriverState::New || driver.state == DriverState::Removed {
            driver
                .apply(DriverOp::Probe, true)
                .map_err(RecoveryPipelineError::Lifecycle)?
        }
        if driver.state == DriverState::Probed {
            driver
                .apply(DriverOp::Init, true)
                .map_err(RecoveryPipelineError::Lifecycle)?
        }
        if driver.state == DriverState::Initialized {
            driver
                .apply(DriverOp::Start, true)
                .map_err(RecoveryPipelineError::Lifecycle)?
        }
        if driver.state != DriverState::Running {
            return Err(RecoveryPipelineError::InconsistentState);
        }
        Ok(())
    }
    pub fn verify_recovery(&mut self, driver: u64) -> Result<(), RecoveryPipelineError> {
        self.record_success(driver)
    }
}
