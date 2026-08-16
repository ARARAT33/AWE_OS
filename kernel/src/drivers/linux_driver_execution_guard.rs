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
        Self { driver_id, state: DriverState::New, health_registered: false }
    }

    pub fn probe(&mut self, resources: &DriverResourceBinding) -> Result<(), ExecutionGuardError> {
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
        if self.state != DriverState::Probed { return Err(ExecutionGuardError::InvalidState); }
        self.state = DriverState::Initialized;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), ExecutionGuardError> {
        if self.state == DriverState::Running { return Err(ExecutionGuardError::AlreadyRunning); }
        if self.state != DriverState::Initialized { return Err(ExecutionGuardError::InvalidState); }
        self.health_registered = true;
        self.state = DriverState::Running;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), ExecutionGuardError> {
        if self.state != DriverState::Running { return Err(ExecutionGuardError::NotRunning); }
        self.health_registered = false;
        self.state = DriverState::Stopped;
        Ok(())
    }

    pub fn remove(&mut self, resources: &mut DriverResourceBinding) -> Result<(), ExecutionGuardError> {
        if self.state == DriverState::Running { return Err(ExecutionGuardError::InvalidState); }
        if self.state != DriverState::Stopped && self.state != DriverState::Initialized && self.state != DriverState::Probed {
            return Err(ExecutionGuardError::InvalidState);
        }
        resources.release_all().map_err(|_| ExecutionGuardError::ResourcesNotOwned)?;
        self.health_registered = false;
        self.state = DriverState::Removed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_is_guarded() {
        let mut g = ExecutionGuard::new(7);
        assert_eq!(g.start(), Err(ExecutionGuardError::InvalidState));
    }

    #[test]
    fn start_registers_health() {
        let mut g = ExecutionGuard::new(7);
        g.state = DriverState::Initialized;
        g.start().unwrap();
        assert_eq!(g.state, DriverState::Running);
        assert!(g.health_registered);
    }

    #[test]
    fn double_start_is_rejected() {
        let mut g = ExecutionGuard::new(7);
        g.state = DriverState::Initialized;
        g.start().unwrap();
        assert_eq!(g.start(), Err(ExecutionGuardError::AlreadyRunning));
    }
}
