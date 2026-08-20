#![no_std]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverOp {
    Probe,
    Init,
    Start,
    Stop,
    Remove,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverOpError {
    InvalidState,
    ProbeFailed,
    InitFailed,
    StartFailed,
    StopFailed,
    RemoveFailed,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverState {
    Empty,
    New,
    Probed,
    Initialized,
    Running,
    Stopped,
    Removed,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverLifecycle {
    pub state: DriverState,
}
impl DriverLifecycle {
    pub const fn new() -> Self {
        Self {
            state: DriverState::New,
        }
    }
    pub fn apply(&mut self, op: DriverOp, success: bool) -> Result<(), DriverOpError> {
        let valid = matches!(
            (self.state, op),
            (DriverState::New, DriverOp::Probe)
                | (DriverState::Probed, DriverOp::Init)
                | (DriverState::Initialized, DriverOp::Start)
                | (DriverState::Running, DriverOp::Stop)
                | (DriverState::Stopped, DriverOp::Remove)
        );
        if !valid {
            return Err(DriverOpError::InvalidState);
        }
        if !success {
            return Err(match op {
                DriverOp::Probe => DriverOpError::ProbeFailed,
                DriverOp::Init => DriverOpError::InitFailed,
                DriverOp::Start => DriverOpError::StartFailed,
                DriverOp::Stop => DriverOpError::StopFailed,
                DriverOp::Remove => DriverOpError::RemoveFailed,
            });
        }
        self.state = match op {
            DriverOp::Probe => DriverState::Probed,
            DriverOp::Init => DriverState::Initialized,
            DriverOp::Start => DriverState::Running,
            DriverOp::Stop => DriverState::Stopped,
            DriverOp::Remove => DriverState::Removed,
        };
        Ok(())
    }
    pub fn rollback_after_failure(&mut self, failed: DriverOp) -> Result<(), DriverOpError> {
        self.state = match failed {
            DriverOp::Probe => DriverState::New,
            DriverOp::Init => DriverState::Probed,
            DriverOp::Start => DriverState::Initialized,
            DriverOp::Stop => DriverState::Running,
            DriverOp::Remove => DriverState::Stopped,
        };
        Ok(())
    }
}

impl Default for DriverLifecycle {
    fn default() -> Self {
        Self::new()
    }
}
