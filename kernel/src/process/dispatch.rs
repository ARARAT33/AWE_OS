#![no_std]
use super::context::CpuContext;
use super::context_switch::SwitchFrame;
use super::x86_64_backend::{BackendError, validate_backend_frames};
use super::{ProcessDescriptor, ProcessId, ProcessState};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DispatchError {
    InvalidProcess,
    NotRunnable,
    Backend(BackendError),
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DispatchTarget {
    pub process: ProcessId,
    pub frame: SwitchFrame,
}
impl DispatchTarget {
    pub fn from_descriptor(
        desc: &ProcessDescriptor,
        context: &CpuContext,
    ) -> Result<Self, DispatchError> {
        if desc.state == ProcessState::Exited || desc.state == ProcessState::Blocked {
            return Err(DispatchError::NotRunnable);
        }
        let frame = SwitchFrame::from_context(context);
        if !frame.is_well_formed() {
            return Err(DispatchError::InvalidProcess);
        }
        Ok(Self {
            process: desc.id,
            frame,
        })
    }
}
pub fn prepare_dispatch(
    current: &SwitchFrame,
    target: &DispatchTarget,
) -> Result<(), DispatchError> {
    validate_backend_frames(current, &target.frame).map_err(DispatchError::Backend)
}
