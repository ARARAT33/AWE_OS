#![no_std]

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceKind {
    Mmio,
    Irq,
    Dma,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Resource {
    pub owner: u64,
    pub kind: ResourceKind,
    pub start: u64,
    pub length: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceError {
    Capacity,
    InvalidRange,
    Overflow,
    Conflict,
    NotFound,
    OwnerMismatch,
}

pub struct ResourceManager<const N: usize> {
    resources: [Option<Resource>; N],
    count: usize,
}

impl<const N: usize> ResourceManager<N> {
    pub const fn new() -> Self {
        Self {
            resources: [None; N],
            count: 0,
        }
    }
    pub const fn count(&self) -> usize {
        self.count
    }
    fn end(start: u64, length: u64) -> Result<u64, ResourceError> {
        if length == 0 {
            return Err(ResourceError::InvalidRange);
        }
        start.checked_add(length).ok_or(ResourceError::Overflow)
    }
    fn overlaps(a: Resource, b: Resource) -> Result<bool, ResourceError> {
        let ae = Self::end(a.start, a.length)?;
        let be = Self::end(b.start, b.length)?;
        Ok(a.kind == b.kind && a.start < be && b.start < ae)
    }
    pub fn acquire(&mut self, resource: Resource) -> Result<usize, ResourceError> {
        Self::end(resource.start, resource.length)?;
        for existing in self.resources.iter().flatten() {
            if Self::overlaps(*existing, resource)? {
                return Err(ResourceError::Conflict);
            }
        }
        let slot = self
            .resources
            .iter()
            .position(Option::is_none)
            .ok_or(ResourceError::Capacity)?;
        self.resources[slot] = Some(resource);
        self.count += 1;
        Ok(slot)
    }
    pub fn release(&mut self, owner: u64, slot: usize) -> Result<Resource, ResourceError> {
        let resource = self
            .resources
            .get(slot)
            .and_then(|r| *r)
            .ok_or(ResourceError::NotFound)?;
        if resource.owner != owner {
            return Err(ResourceError::OwnerMismatch);
        }
        self.resources[slot] = None;
        self.count -= 1;
        Ok(resource)
    }
    pub fn release_owner(&mut self, owner: u64) -> usize {
        let mut removed = 0;
        for entry in &mut self.resources {
            if entry.map(|r| r.owner == owner).unwrap_or(false) {
                *entry = None;
                self.count -= 1;
                removed += 1;
            }
        }
        removed
    }
    pub fn get(&self, slot: usize) -> Option<Resource> {
        self.resources.get(slot).and_then(|r| *r)
    }
}

impl<const N: usize> Default for ResourceManager<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_overlapping_mmio() {
        let mut m = ResourceManager::<4>::new();
        m.acquire(Resource {
            owner: 1,
            kind: ResourceKind::Mmio,
            start: 0x1000,
            length: 0x100,
        })
        .unwrap();
        assert_eq!(
            m.acquire(Resource {
                owner: 2,
                kind: ResourceKind::Mmio,
                start: 0x1080,
                length: 0x20
            }),
            Err(ResourceError::Conflict)
        );
    }
    #[test]
    fn permits_different_resource_kinds() {
        let mut m = ResourceManager::<4>::new();
        m.acquire(Resource {
            owner: 1,
            kind: ResourceKind::Mmio,
            start: 0x1000,
            length: 0x100,
        })
        .unwrap();
        m.acquire(Resource {
            owner: 1,
            kind: ResourceKind::Irq,
            start: 5,
            length: 1,
        })
        .unwrap();
        assert_eq!(m.count(), 2);
    }
    #[test]
    fn owner_cleanup_is_bounded() {
        let mut m = ResourceManager::<4>::new();
        m.acquire(Resource {
            owner: 7,
            kind: ResourceKind::Dma,
            start: 1,
            length: 1,
        })
        .unwrap();
        m.acquire(Resource {
            owner: 8,
            kind: ResourceKind::Dma,
            start: 2,
            length: 1,
        })
        .unwrap();
        assert_eq!(m.release_owner(7), 1);
        assert_eq!(m.count(), 1);
    }
}
