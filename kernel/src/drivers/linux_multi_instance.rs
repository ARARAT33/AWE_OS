#![no_std]

use super::linux_driver_ops::{DriverLifecycle, DriverOp, DriverOpError, DriverState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MultiInstanceError { Full, Invalid, Lifecycle(DriverOpError) }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverInstance {
    pub id: u64,
    pub lifecycle: DriverLifecycle,
    pub active: bool,
}

impl DriverInstance {
    pub const fn empty() -> Self { Self { id: 0, lifecycle: DriverLifecycle::new(), active: false } }
    pub const fn new(id: u64) -> Self { Self { id, lifecycle: DriverLifecycle::new(), active: false } }
}

pub struct MultiInstanceManager<const N: usize> {
    pub instances: [DriverInstance; N],
    pub count: usize,
}

impl<const N: usize> MultiInstanceManager<N> {
    pub const fn new() -> Self { Self { instances: [DriverInstance::empty(); N], count: 0 } }

    pub fn add(&mut self, id: u64) -> Result<usize, MultiInstanceError> {
        if self.count == N || self.instances[..self.count].iter().any(|x| x.id == id) { return Err(if self.count == N { MultiInstanceError::Full } else { MultiInstanceError::Invalid }); }
        let index = self.count; self.instances[index] = DriverInstance::new(id); self.count += 1; Ok(index)
    }

    pub fn probe(&mut self, index: usize) -> Result<(), MultiInstanceError> { self.apply(index, DriverOp::Probe, true) }
    pub fn init(&mut self, index: usize) -> Result<(), MultiInstanceError> { self.apply(index, DriverOp::Init, true) }
    pub fn start(&mut self, index: usize) -> Result<(), MultiInstanceError> { self.apply(index, DriverOp::Start, true) }

    pub fn stop(&mut self, index: usize) -> Result<(), MultiInstanceError> { self.apply(index, DriverOp::Stop, true) }
    pub fn remove(&mut self, index: usize) -> Result<(), MultiInstanceError> { self.apply(index, DriverOp::Remove, true) }

    pub fn rollback_instance(&mut self, index: usize) -> Result<(), MultiInstanceError> {
        let state = self.instance(index)?.lifecycle.state;
        match state {
            DriverState::Running => self.stop(index)?,
            DriverState::Stopped => {},
            DriverState::Initialized => {},
            DriverState::Probed => {},
            DriverState::Removed => return Ok(()),
            DriverState::Empty => return Err(MultiInstanceError::Invalid),
        }
        let state = self.instance(index)?.lifecycle.state;
        if state != DriverState::Removed { self.remove(index)?; }
        self.instance_mut(index)?.active = false;
        Ok(())
    }

    pub fn rollback_all(&mut self) -> Result<(), MultiInstanceError> {
        let mut i = self.count;
        while i > 0 { i -= 1; if self.instances[i].active { self.rollback_instance(i)?; } }
        Ok(())
    }

    pub fn activate_all(&mut self) -> Result<(), MultiInstanceError> {
        let mut i = 0;
        while i < self.count {
            self.probe(i)?; self.init(i)?; self.start(i)?; self.instances[i].active = true; i += 1;
        }
        Ok(())
    }

    pub fn instance(&self, index: usize) -> Result<&DriverInstance, MultiInstanceError> { self.instances.get(index).ok_or(MultiInstanceError::Invalid) }
    fn instance_mut(&mut self, index: usize) -> Result<&mut DriverInstance, MultiInstanceError> { self.instances.get_mut(index).ok_or(MultiInstanceError::Invalid) }
    fn apply(&mut self, index: usize, op: DriverOp, success: bool) -> Result<(), MultiInstanceError> { self.instance_mut(index)?.lifecycle.apply(op, success).map_err(MultiInstanceError::Lifecycle) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn activates_multiple_instances_and_rolls_back_in_reverse_order() {
        let mut m = MultiInstanceManager::<3>::new(); m.add(10).unwrap(); m.add(20).unwrap(); m.add(30).unwrap();
        m.activate_all().unwrap();
        assert_eq!(m.instance(0).unwrap().lifecycle.state, DriverState::Running);
        assert_eq!(m.instance(2).unwrap().lifecycle.state, DriverState::Running);
        m.rollback_all().unwrap();
        for i in 0..3 { assert_eq!(m.instance(i).unwrap().lifecycle.state, DriverState::Removed); assert!(!m.instance(i).unwrap().active); }
    }
    #[test]
    fn duplicate_ids_are_rejected() { let mut m=MultiInstanceManager::<2>::new(); m.add(1).unwrap(); assert_eq!(m.add(1),Err(MultiInstanceError::Invalid)); }
}
