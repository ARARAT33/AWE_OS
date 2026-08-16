#![no_std]

use super::linux_driver_ops::{DriverLifecycle, DriverOp, DriverOpError, DriverState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegistryError {
    Full,
    Duplicate,
    NotFound,
    InvalidId,
    Lifecycle(DriverOpError),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverRecord {
    pub id: u64,
    pub class: u16,
    pub vendor: u16,
    pub device: u16,
    pub lifecycle: DriverState,
    pub enabled: bool,
}

impl DriverRecord {
    pub const EMPTY: Self = Self {
        id: 0,
        class: 0,
        vendor: 0,
        device: 0,
        lifecycle: DriverState::New,
        enabled: false,
    };
}

pub struct DriverRegistry<const N: usize> {
    records: [DriverRecord; N],
    used: usize,
}

impl<const N: usize> DriverRegistry<N> {
    pub const fn new() -> Self {
        Self { records: [DriverRecord::EMPTY; N], used: 0 }
    }

    pub const fn len(&self) -> usize { self.used }
    pub const fn is_empty(&self) -> bool { self.used == 0 }

    pub fn register(&mut self, id: u64, class: u16, vendor: u16, device: u16) -> Result<usize, RegistryError> {
        if id == 0 { return Err(RegistryError::InvalidId); }
        if self.find_index(id).is_some() { return Err(RegistryError::Duplicate); }
        if self.used == N { return Err(RegistryError::Full); }
        let index = self.used;
        self.records[index] = DriverRecord {
            id,
            class,
            vendor,
            device,
            lifecycle: DriverState::New,
            enabled: true,
        };
        self.used += 1;
        Ok(index)
    }

    pub fn unregister(&mut self, id: u64) -> Result<DriverRecord, RegistryError> {
        let index = self.find_index(id).ok_or(RegistryError::NotFound)?;
        let old = self.records[index];
        if old.lifecycle != DriverState::Removed && old.lifecycle != DriverState::New {
            return Err(RegistryError::Lifecycle(DriverOpError::InvalidState));
        }
        let last = self.used - 1;
        if index != last { self.records[index] = self.records[last]; }
        self.records[last] = DriverRecord::EMPTY;
        self.used -= 1;
        Ok(old)
    }

    pub fn apply(&mut self, id: u64, op: DriverOp, success: bool) -> Result<DriverState, RegistryError> {
        let index = self.find_index(id).ok_or(RegistryError::NotFound)?;
        let record = &mut self.records[index];
        let mut lifecycle = DriverLifecycle::new();
        lifecycle.state = record.lifecycle;
        lifecycle.apply(op, success).map_err(RegistryError::Lifecycle)?;
        record.lifecycle = lifecycle.state;
        if record.lifecycle == DriverState::Removed { record.enabled = false; }
        Ok(record.lifecycle)
    }

    pub fn set_enabled(&mut self, id: u64, enabled: bool) -> Result<(), RegistryError> {
        let index = self.find_index(id).ok_or(RegistryError::NotFound)?;
        self.records[index].enabled = enabled;
        Ok(())
    }

    pub fn get(&self, id: u64) -> Result<DriverRecord, RegistryError> {
        let index = self.find_index(id).ok_or(RegistryError::NotFound)?;
        Ok(self.records[index])
    }

    pub fn get_at(&self, index: usize) -> Result<DriverRecord, RegistryError> {
        if index >= self.used { return Err(RegistryError::NotFound); }
        Ok(self.records[index])
    }

    fn find_index(&self, id: u64) -> Option<usize> {
        let mut i = 0;
        while i < self.used {
            if self.records[i].id == id { return Some(i); }
            i += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_runs_lifecycle() {
        let mut r = DriverRegistry::<2>::new();
        r.register(10, 1, 2, 3).unwrap();
        assert_eq!(r.apply(10, DriverOp::Probe, true).unwrap(), DriverState::Probed);
        assert_eq!(r.apply(10, DriverOp::Init, true).unwrap(), DriverState::Initialized);
        assert_eq!(r.apply(10, DriverOp::Start, true).unwrap(), DriverState::Running);
        assert!(r.get(10).unwrap().enabled);
    }

    #[test]
    fn rejects_duplicate_and_full_registry() {
        let mut r = DriverRegistry::<1>::new();
        r.register(10, 1, 2, 3).unwrap();
        assert_eq!(r.register(10, 1, 2, 3), Err(RegistryError::Duplicate));
        assert_eq!(r.register(11, 1, 2, 4), Err(RegistryError::Full));
    }

    #[test]
    fn unregister_requires_safe_state() {
        let mut r = DriverRegistry::<2>::new();
        r.register(10, 1, 2, 3).unwrap();
        assert!(matches!(r.unregister(10), Err(RegistryError::Lifecycle(_))));
        r.apply(10, DriverOp::Probe, true).unwrap();
        r.apply(10, DriverOp::Init, true).unwrap();
        r.apply(10, DriverOp::Start, true).unwrap();
        r.apply(10, DriverOp::Stop, true).unwrap();
        r.apply(10, DriverOp::Remove, true).unwrap();
        r.unregister(10).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn disabled_removed_driver_is_not_reported_enabled() {
        let mut r = DriverRegistry::<1>::new();
        r.register(7, 9, 10, 11).unwrap();
        for op in [DriverOp::Probe, DriverOp::Init, DriverOp::Start, DriverOp::Stop, DriverOp::Remove] {
            r.apply(7, op, true).unwrap();
        }
        assert!(!r.get(7).unwrap().enabled);
    }
}
