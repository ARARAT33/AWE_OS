#![no_std]

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverOp { Probe, Init, Start, Stop, Remove }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverOpError { InvalidState, ProbeFailed, InitFailed, StartFailed, StopFailed, RemoveFailed }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DriverState { New, Probed, Initialized, Running, Stopped, Removed }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverLifecycle {
    pub state: DriverState,
}

impl DriverLifecycle {
    pub const fn new() -> Self { Self { state: DriverState::New } }

    pub fn apply(&mut self, op: DriverOp, success: bool) -> Result<(), DriverOpError> {
        let valid = match (self.state, op) {
            (DriverState::New, DriverOp::Probe) => true,
            (DriverState::Probed, DriverOp::Init) => true,
            (DriverState::Initialized, DriverOp::Start) => true,
            (DriverState::Running, DriverOp::Stop) => true,
            (DriverState::Stopped, DriverOp::Remove) => true,
            _ => false,
        };
        if !valid { return Err(DriverOpError::InvalidState); }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_reaches_running() {
        let mut d = DriverLifecycle::new();
        d.apply(DriverOp::Probe, true).unwrap();
        d.apply(DriverOp::Init, true).unwrap();
        d.apply(DriverOp::Start, true).unwrap();
        assert_eq!(d.state, DriverState::Running);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut d = DriverLifecycle::new();
        assert_eq!(d.apply(DriverOp::Start, true), Err(DriverOpError::InvalidState));
    }

    #[test]
    fn failed_start_rolls_back_to_initialized() {
        let mut d = DriverLifecycle::new();
        d.apply(DriverOp::Probe, true).unwrap();
        d.apply(DriverOp::Init, true).unwrap();
        assert_eq!(d.apply(DriverOp::Start, false), Err(DriverOpError::StartFailed));
        d.rollback_after_failure(DriverOp::Start).unwrap();
        assert_eq!(d.state, DriverState::Initialized);
    }
}
