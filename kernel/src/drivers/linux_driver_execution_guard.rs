#![no_std]
use super::linux_driver_ops::DriverState;
use super::linux_driver_resource_binding::DriverResourceBinding;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExecutionGuardError {
    InvalidState,
    ResourcesNotOwned,
    AlreadyRunning,
    NotRunning,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ExecutionGuard {
    pub driver_id: u64,
    pub state: DriverState,
    pub health_registered: bool,
}
impl ExecutionGuard {
    pub const fn new(driver_id: u64) -> Self {
        Self {
            driver_id,
            state: DriverState::New,
            health_registered: false,
        }
    }
    pub fn probe<const N: usize>(
        &mut self,
        resources: &DriverResourceBinding<N>,
    ) -> Result<(), ExecutionGuardError> {
        if self.state != DriverState::New && self.state != DriverState::Removed {
            return Err(ExecutionGuardError::InvalidState);
        }
        if resources.owner() != self.driver_id || !resources.has_resources() {
            return Err(ExecutionGuardError::ResourcesNotOwned);
        }
        self.state = DriverState::Probed;
        Ok(())
    }
    pub fn init(&mut self) -> Result<(), ExecutionGuardError> {
        if self.state != DriverState::Probed {
            return Err(ExecutionGuardError::InvalidState);
        }
        self.state = DriverState::Initialized;
        Ok(())
    }
    pub fn start(&mut self) -> Result<(), ExecutionGuardError> {
        if self.state == DriverState::Running {
            return Err(ExecutionGuardError::AlreadyRunning);
        }
        if self.state != DriverState::Initialized {
            return Err(ExecutionGuardError::InvalidState);
        }
        self.health_registered = true;
        self.state = DriverState::Running;
        Ok(())
    }
    pub fn stop(&mut self) -> Result<(), ExecutionGuardError> {
        if self.state != DriverState::Running {
            return Err(ExecutionGuardError::NotRunning);
        }
        self.health_registered = false;
        self.state = DriverState::Stopped;
        Ok(())
    }
    pub fn remove<const N: usize>(
        &mut self,
        resources: &mut DriverResourceBinding<N>,
    ) -> Result<(), ExecutionGuardError> {
        if self.state == DriverState::Running {
            return Err(ExecutionGuardError::InvalidState);
        }
        if !matches!(
            self.state,
            DriverState::Stopped | DriverState::Initialized | DriverState::Probed
        ) {
            return Err(ExecutionGuardError::InvalidState);
        }
        resources.release_all();
        self.health_registered = false;
        self.state = DriverState::Removed;
        Ok(())
    }
}
