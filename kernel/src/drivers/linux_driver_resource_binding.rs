#![no_std]

use super::linux_driver_ops::{DriverLifecycle, DriverOp, DriverOpError, DriverState};
use super::linux_resource_manager::{Resource, ResourceError, ResourceManager};
use super::linux_resource_transaction::{ResourceTransaction, ResourceTransactionError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BindingError {
    Resource(ResourceError),
    Transaction(ResourceTransactionError),
    Lifecycle(DriverOpError),
    InvalidState,
}

pub struct DriverResourceBinding<const N: usize = 16> {
    pub owner: u64,
    pub resources: ResourceManager<N>,
    active: bool,
}

impl<const N: usize> DriverResourceBinding<N> {
    pub const fn new(owner: u64) -> Self {
        Self {
            owner,
            resources: ResourceManager::new(),
            active: false,
        }
    }
    pub const fn owner(&self) -> u64 {
        self.owner
    }
    pub const fn has_resources(&self) -> bool {
        self.resources.count() != 0
    }
    pub fn acquire_all(&mut self, requested: &[Resource]) -> Result<(), BindingError> {
        if self.active || self.has_resources() {
            return Err(BindingError::InvalidState);
        }
        let mut tx = ResourceTransaction::begin(&mut self.resources);
        for resource in requested.iter().copied() {
            if resource.owner != self.owner {
                let _ = tx.rollback();
                return Err(BindingError::InvalidState);
            }
            tx.acquire(resource).map_err(BindingError::Transaction)?;
        }
        tx.commit().map_err(BindingError::Transaction)?;
        self.active = true;
        Ok(())
    }
    pub fn probe_and_init(&mut self, driver: &mut DriverLifecycle) -> Result<(), BindingError> {
        if !self.active || driver.state != DriverState::New {
            return Err(BindingError::InvalidState);
        }
        driver
            .apply(DriverOp::Probe, true)
            .map_err(BindingError::Lifecycle)?;
        if let Err(error) = driver.apply(DriverOp::Init, true) {
            let _ = driver.apply(DriverOp::Remove, true);
            self.release_all();
            return Err(BindingError::Lifecycle(error));
        }
        Ok(())
    }
    pub fn start(&mut self, driver: &mut DriverLifecycle) -> Result<(), BindingError> {
        if !self.active || driver.state != DriverState::Initialized {
            return Err(BindingError::InvalidState);
        }
        if let Err(error) = driver.apply(DriverOp::Start, true) {
            let _ = driver.apply(DriverOp::Remove, true);
            self.release_all();
            return Err(BindingError::Lifecycle(error));
        }
        Ok(())
    }
    pub fn stop(&mut self, driver: &mut DriverLifecycle) -> Result<(), BindingError> {
        if driver.state == DriverState::Running {
            driver
                .apply(DriverOp::Stop, true)
                .map_err(BindingError::Lifecycle)?;
        }
        if matches!(
            driver.state,
            DriverState::Stopped | DriverState::Initialized | DriverState::Probed
        ) {
            driver
                .apply(DriverOp::Remove, true)
                .map_err(BindingError::Lifecycle)?;
        }
        self.release_all();
        Ok(())
    }
    pub fn release_all(&mut self) -> usize {
        let released = self.resources.release_owner(self.owner);
        self.active = false;
        released
    }
    pub const fn active(&self) -> bool {
        self.active
    }
    pub const fn resource_count(&self) -> usize {
        self.resources.count()
    }
}

#[cfg(test)]
mod tests {
    use super::super::linux_resource_manager::ResourceKind;
    use super::*;
    #[test]
    fn failed_second_resource_rolls_back_first() {
        let mut b = DriverResourceBinding::<4>::new(10);
        let r = [
            Resource {
                owner: 10,
                kind: ResourceKind::Mmio,
                start: 0x1000,
                length: 0x100,
            },
            Resource {
                owner: 10,
                kind: ResourceKind::Mmio,
                start: 0x1080,
                length: 0x20,
            },
        ];
        assert!(b.acquire_all(&r).is_err());
        assert_eq!(b.resource_count(), 0)
    }
}
