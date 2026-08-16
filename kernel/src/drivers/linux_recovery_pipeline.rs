#![no_std]

use super::linux_driver_health::{DriverHealthMonitor, HealthError, HealthState};
use super::linux_fault_recovery::{FaultRecovery, RecoveryError};
use super::linux_fault_impact::{FaultImpact, FaultImpactError};
use super::linux_dependency_multi_instance::DependencyMultiError;
use super::linux_driver_ops::{DriverLifecycle, DriverOp, DriverOpError, DriverState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecoveryPipelineError { Health(HealthError), Impact(FaultImpactError), Recovery(RecoveryError), Dependency(DependencyMultiError), Lifecycle(DriverOpError), Capacity, InconsistentState }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RecoveryReport<const N: usize> { pub failed_driver: u64, pub affected: [u64; N], pub affected_count: usize, pub recovered: bool, pub quarantined: bool }

pub struct RecoveryPipeline<const N: usize> { pub health: DriverHealthMonitor<N>, pub impact: FaultImpact<N>, pub recovery: FaultRecovery<N> }

impl<const N: usize> RecoveryPipeline<N> {
    pub const fn new() -> Self { Self { health: DriverHealthMonitor::new(), impact: FaultImpact::new(), recovery: FaultRecovery::new(3) } }
    pub fn record_success(&mut self, driver: u64) -> Result<(), RecoveryPipelineError> { self.health.record_success(driver).map_err(RecoveryPipelineError::Health) }
    pub fn record_failure(&mut self, driver: u64) -> Result<HealthState, RecoveryPipelineError> { self.health.record_failure(driver).map_err(RecoveryPipelineError::Health) }
    pub fn analyze(&mut self, failed: u64, edges: &[(u64, u64)]) -> Result<usize, RecoveryPipelineError> { self.impact.compute(failed, edges).map_err(RecoveryPipelineError::Impact) }
    pub fn recover_instance(&mut self, driver: &mut DriverLifecycle) -> Result<(), RecoveryPipelineError> {
        if driver.state == DriverState::Running { driver.apply(DriverOp::Stop, true).map_err(RecoveryPipelineError::Lifecycle)?; }
        if driver.state == DriverState::Stopped { driver.apply(DriverOp::Remove, true).map_err(RecoveryPipelineError::Lifecycle)?; }
        if driver.state == DriverState::New || driver.state == DriverState::Removed { driver.apply(DriverOp::Probe, true).map_err(RecoveryPipelineError::Lifecycle)?; }
        if driver.state == DriverState::Probed { driver.apply(DriverOp::Init, true).map_err(RecoveryPipelineError::Lifecycle)?; }
        if driver.state == DriverState::Initialized { driver.apply(DriverOp::Start, true).map_err(RecoveryPipelineError::Lifecycle)?; }
        if driver.state != DriverState::Running { return Err(RecoveryPipelineError::InconsistentState); }
        Ok(())
    }
    pub fn verify_recovery(&mut self, driver: u64) -> Result<(), RecoveryPipelineError> { self.health.record_success(driver).map_err(RecoveryPipelineError::Health) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn pipeline_recovers_running_driver() { let mut p=RecoveryPipeline::<4>::new(); let mut l=DriverLifecycle::new(); l.apply(DriverOp::Probe,true).unwrap(); l.apply(DriverOp::Init,true).unwrap(); l.apply(DriverOp::Start,true).unwrap(); p.record_failure(7).unwrap(); p.analyze(7,&[(7,8),(8,9)]).unwrap(); p.recover_instance(&mut l).unwrap(); p.verify_recovery(7).unwrap(); assert_eq!(l.state,DriverState::Running); }
    #[test] fn failure_chain_is_tracked() { let mut p=RecoveryPipeline::<4>::new(); assert_eq!(p.record_failure(1).unwrap(),HealthState::Faulted); assert_eq!(p.record_failure(1).unwrap(),HealthState::Faulted); assert_eq!(p.record_failure(1).unwrap(),HealthState::Quarantined); }
}
