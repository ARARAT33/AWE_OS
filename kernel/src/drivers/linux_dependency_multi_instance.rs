#![no_std]

use super::linux_dependency::Dependency;
use super::linux_dependency_order::{topological_order, OrderError};
use super::linux_multi_instance::{MultiInstanceError, MultiInstanceManager};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DependencyMultiError { Order(OrderError), Instance(MultiInstanceError), MissingNode }

/// Dependency-aware coordinator for multiple driver instances.
/// `ids` are driver module hashes; dependencies must be activated first.
pub struct DependencyMultiInstanceManager<const N: usize> {
    pub manager: MultiInstanceManager<N>,
    pub order: [u64; N],
    pub order_len: usize,
}

impl<const N: usize> DependencyMultiInstanceManager<N> {
    pub const fn new() -> Self { Self { manager: MultiInstanceManager::new(), order: [0; N], order_len: 0 } }

    pub fn add(&mut self, id: u64) -> Result<usize, DependencyMultiError> {
        self.manager.add(id).map_err(DependencyMultiError::Instance)
    }

    pub fn prepare_dependencies(&mut self, deps: &[Dependency]) -> Result<(), DependencyMultiError> {
        let mut i = 0;
        while i < self.manager.count { self.order[i] = self.manager.instances[i].id; i += 1; }
        self.order_len = topological_order(&self.order[..self.manager.count], deps, &mut self.order)
            .map_err(DependencyMultiError::Order)?;
        Ok(())
    }

    pub fn activate_in_dependency_order(&mut self) -> Result<(), DependencyMultiError> {
        let mut pos = 0;
        while pos < self.order_len {
            let id = self.order[pos];
            let mut index = None;
            let mut i = 0;
            while i < self.manager.count { if self.manager.instances[i].id == id { index = Some(i); break; } i += 1; }
            let index = index.ok_or(DependencyMultiError::MissingNode)?;
            if self.manager.instances[index].active { pos += 1; continue; }
            if let Err(e) = self.manager.probe(index).and_then(|_| self.manager.init(index)).and_then(|_| self.manager.start(index)) {
                let _ = self.rollback_active();
                return Err(DependencyMultiError::Instance(e));
            }
            self.manager.instances[index].active = true;
            pos += 1;
        }
        Ok(())
    }

    pub fn rollback_active(&mut self) -> Result<(), DependencyMultiError> {
        let mut pos = self.order_len;
        while pos > 0 { pos -= 1;
            let id = self.order[pos];
            let mut i = 0;
            while i < self.manager.count { if self.manager.instances[i].id == id && self.manager.instances[i].active { self.manager.rollback_instance(i).map_err(DependencyMultiError::Instance)?; break; } i += 1; }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependency_order_is_used_for_activation_and_rollback() {
        let mut m = DependencyMultiInstanceManager::<3>::new();
        m.add(10).unwrap(); m.add(20).unwrap(); m.add(30).unwrap();
        let deps = [Dependency { driver_hash: 10, required_hash: 20 }, Dependency { driver_hash: 20, required_hash: 30 }];
        m.prepare_dependencies(&deps).unwrap();
        assert_eq!(&m.order[..m.order_len], &[30, 20, 10]);
        m.activate_in_dependency_order().unwrap();
        m.rollback_active().unwrap();
        for i in 0..3 { assert!(!m.manager.instances[i].active); }
    }
}
