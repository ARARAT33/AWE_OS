#![no_std]
use super::linux_driver_ops::{DriverLifecycle, DriverOp, DriverOpError, DriverState};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MultiInstanceError {
    Full,
    Invalid,
    Lifecycle(DriverOpError),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverInstance {
    pub id: u64,
    pub lifecycle: DriverLifecycle,
    pub active: bool,
}
impl DriverInstance {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            lifecycle: DriverLifecycle::new(),
            active: false,
        }
    }
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            lifecycle: DriverLifecycle::new(),
            active: false,
        }
    }
}
pub struct MultiInstanceManager<const N: usize> {
    pub instances: [DriverInstance; N],
    pub count: usize,
}
impl<const N: usize> MultiInstanceManager<N> {
    pub const fn new() -> Self {
        Self {
            instances: [DriverInstance::empty(); N],
            count: 0,
        }
    }
    pub fn add(&mut self, id: u64) -> Result<usize, MultiInstanceError> {
        if self.count == N || self.instances[..self.count].iter().any(|x| x.id == id) {
            return Err(if self.count == N {
                MultiInstanceError::Full
            } else {
                MultiInstanceError::Invalid
            });
        }
        let i = self.count;
        self.instances[i] = DriverInstance::new(id);
        self.count += 1;
        Ok(i)
    }
    pub fn probe(&mut self, index: usize) -> Result<(), MultiInstanceError> {
        self.apply(index, DriverOp::Probe, true)
    }
    pub fn init(&mut self, index: usize) -> Result<(), MultiInstanceError> {
        self.apply(index, DriverOp::Init, true)
    }
    pub fn start(&mut self, index: usize) -> Result<(), MultiInstanceError> {
        self.apply(index, DriverOp::Start, true)?;
        self.instance_mut(index)?.active = true;
        Ok(())
    }
    pub fn stop(&mut self, index: usize) -> Result<(), MultiInstanceError> {
        self.apply(index, DriverOp::Stop, true)?;
        self.instance_mut(index)?.active = false;
        Ok(())
    }
    pub fn remove(&mut self, index: usize) -> Result<(), MultiInstanceError> {
        self.apply(index, DriverOp::Remove, true)?;
        self.instance_mut(index)?.active = false;
        Ok(())
    }
    pub fn rollback_instance(&mut self, index: usize) -> Result<(), MultiInstanceError> {
        let state = self.instance(index)?.lifecycle.state;
        match state {
            DriverState::Empty => return Err(MultiInstanceError::Invalid),
            DriverState::New => return Ok(()),
            DriverState::Running => self.stop(index)?,
            DriverState::Stopped | DriverState::Initialized | DriverState::Probed => {}
            DriverState::Removed => return Ok(()),
        }
        let state = self.instance(index)?.lifecycle.state;
        if state != DriverState::Removed {
            self.remove(index)?
        }
        let instance = self.instance_mut(index)?;
        instance.lifecycle = DriverLifecycle::new();
        instance.active = false;
        Ok(())
    }
    pub fn rollback_all(&mut self) -> Result<(), MultiInstanceError> {
        let mut i = self.count;
        while i > 0 {
            i -= 1;
            if self.instances[i].active {
                self.rollback_instance(i)?
            }
        }
        Ok(())
    }
    pub fn activate_all(&mut self) -> Result<(), MultiInstanceError> {
        let mut i = 0;
        while i < self.count {
            self.probe(i)?;
            self.init(i)?;
            self.start(i)?;
            i += 1
        }
        Ok(())
    }
    pub fn instance(&self, index: usize) -> Result<&DriverInstance, MultiInstanceError> {
        self.instances.get(index).ok_or(MultiInstanceError::Invalid)
    }
    fn instance_mut(&mut self, index: usize) -> Result<&mut DriverInstance, MultiInstanceError> {
        self.instances
            .get_mut(index)
            .ok_or(MultiInstanceError::Invalid)
    }
    fn apply(
        &mut self,
        index: usize,
        op: DriverOp,
        success: bool,
    ) -> Result<(), MultiInstanceError> {
        self.instance_mut(index)?
            .lifecycle
            .apply(op, success)
            .map_err(MultiInstanceError::Lifecycle)
    }
}
