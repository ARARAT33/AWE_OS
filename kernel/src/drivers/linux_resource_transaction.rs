#![no_std]

use super::linux_resource_manager::{Resource, ResourceError, ResourceManager};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceTransactionError {
    Resource(ResourceError),
    Capacity,
    NotActive,
}

pub struct ResourceTransaction<'a, const N: usize> {
    manager: &'a mut ResourceManager<N>,
    acquired: [Option<(usize, u64)>; N],
    count: usize,
    active: bool,
}

impl<'a, const N: usize> ResourceTransaction<'a, N> {
    pub fn begin(manager: &'a mut ResourceManager<N>) -> Self {
        Self {
            manager,
            acquired: [None; N],
            count: 0,
            active: true,
        }
    }

    pub fn acquire(&mut self, resource: Resource) -> Result<usize, ResourceTransactionError> {
        if !self.active {
            return Err(ResourceTransactionError::NotActive);
        }
        if self.count == N {
            return Err(ResourceTransactionError::Capacity);
        }
        let slot = self
            .manager
            .acquire(resource)
            .map_err(ResourceTransactionError::Resource)?;
        self.acquired[self.count] = Some((slot, resource.owner));
        self.count += 1;
        Ok(slot)
    }

    pub fn commit(mut self) -> Result<(), ResourceTransactionError> {
        if !self.active {
            return Err(ResourceTransactionError::NotActive);
        }
        self.active = false;
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<usize, ResourceTransactionError> {
        if !self.active {
            return Err(ResourceTransactionError::NotActive);
        }
        let mut released = 0;
        while self.count != 0 {
            self.count -= 1;
            if let Some((slot, owner)) = self.acquired[self.count].take() {
                self.manager
                    .release(owner, slot)
                    .map_err(ResourceTransactionError::Resource)?;
                released += 1;
            }
        }
        self.active = false;
        Ok(released)
    }
}

impl<'a, const N: usize> Drop for ResourceTransaction<'a, N> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        while self.count != 0 {
            self.count -= 1;
            if let Some((slot, owner)) = self.acquired[self.count].take() {
                let _ = self.manager.release(owner, slot);
            }
        }
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::super::linux_resource_manager::ResourceKind;
    use super::*;

    #[test]
    fn failed_transaction_rolls_back_all_resources() {
        let mut manager = ResourceManager::<4>::new();
        {
            let mut tx = ResourceTransaction::begin(&mut manager);
            tx.acquire(Resource {
                owner: 1,
                kind: ResourceKind::Mmio,
                start: 0x1000,
                length: 0x100,
            })
            .unwrap();
            tx.acquire(Resource {
                owner: 1,
                kind: ResourceKind::Irq,
                start: 5,
                length: 1,
            })
            .unwrap();
            assert_eq!(tx.rollback().unwrap(), 2);
        }
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn drop_is_a_safety_rollback() {
        let mut manager = ResourceManager::<2>::new();
        {
            let mut tx = ResourceTransaction::begin(&mut manager);
            tx.acquire(Resource {
                owner: 4,
                kind: ResourceKind::Dma,
                start: 10,
                length: 2,
            })
            .unwrap();
        }
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn commit_keeps_resources() {
        let mut manager = ResourceManager::<2>::new();
        let mut tx = ResourceTransaction::begin(&mut manager);
        tx.acquire(Resource {
            owner: 9,
            kind: ResourceKind::Mmio,
            start: 0x2000,
            length: 0x20,
        })
        .unwrap();
        tx.commit().unwrap();
        assert_eq!(manager.count(), 1);
    }
}
