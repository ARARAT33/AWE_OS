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

pub struct DriverResourceBinding<const N: usize> {
    pub owner: u64,
    pub resources: ResourceManager<N>,
    active: bool,
}

impl<const N: usize> DriverResourceBinding<N> {
    pub const fn new(owner: u64) -> Self { Self { owner, resources: ResourceManager::new(), active: false } }
    pub const fn owner(&self) -> u64 { self.owner }
    pub const fn has_resources(&self) -> bool { self.resources.count() != 0 }

    pub fn acquire_all(&mut self, requested: &[Resource]) -> Result<(), BindingError> {
        let mut tx = ResourceTransaction::begin(&mut self.resources);
        for resource in requested.iter().copied() {
            if resource.owner != self.owner { let _ = tx.rollback(); return Err(BindingError::InvalidState); }
            tx.acquire(resource).map_err(BindingError::Transaction)?;
        }
        tx.commit().map_err(BindingError::Transaction)?;
        self.active = true;
        Ok(())
    }

    pub fn start(&mut self, driver: &mut DriverLifecycle) -> Result<(), BindingError> {
        if !self.active || driver.state != DriverState::Initialized { return Err(BindingError::InvalidState); }
        driver.apply(DriverOp::Start, true).map_err(BindingError::Lifecycle)
    }

    pub fn stop(&mut self, driver: &mut DriverLifecycle) -> Result<(), BindingError> {
        if driver.state == DriverState::Running { driver.apply(DriverOp::Stop, true).map_err(BindingError::Lifecycle)?; }
        self.release_all();
        Ok(())
    }

    pub fn release_all(&mut self) -> usize { let released = self.resources.release_owner(self.owner); self.active = false; released }
    pub const fn active(&self) -> bool { self.active }
    pub const fn resource_count(&self) -> usize { self.resources.count() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::linux_resource_manager::ResourceKind;

    #[test] fn failed_second_resource_rolls_back_first() {
        let mut binding = DriverResourceBinding::<4>::new(10);
        let requested = [Resource { owner: 10, kind: ResourceKind::Mmio, start: 0x1000, length: 0x100 }, Resource { owner: 10, kind: ResourceKind::Mmio, start: 0x1080, length: 0x20 }];
        assert!(binding.acquire_all(&requested).is_err()); assert_eq!(binding.resource_count(), 0); assert!(!binding.active());
    }

    #[test] fn rejects_wrong_owner_without_leaking() {
        let mut binding = DriverResourceBinding::<4>::new(10);
        let requested = [Resource { owner: 11, kind: ResourceKind::Irq, start: 11, length: 1 }];
        assert_eq!(binding.acquire_all(&requested), Err(BindingError::InvalidState)); assert_eq!(binding.resource_count(), 0);
    }

    #[test] fn successful_binding_can_be_released() {
        let mut binding = DriverResourceBinding::<4>::new(7);
        let requested = [Resource { owner: 7, kind: ResourceKind::Irq, start: 11, length: 1 }];
        binding.acquire_all(&requested).unwrap(); assert!(binding.active()); assert_eq!(binding.release_all(), 1); assert_eq!(binding.resource_count(), 0);
    }
}
