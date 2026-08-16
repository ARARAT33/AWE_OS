#![no_std]

use super::{ProcessDescriptor, ProcessId, ProcessState};
use super::context::CpuContext;
use super::context_switch::SwitchFrame;
use super::x86_64_backend::{validate_backend_frames, BackendError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DispatchError { InvalidProcess, NotRunnable, Backend(BackendError) }

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DispatchTarget { pub process: ProcessId, pub frame: SwitchFrame }

impl DispatchTarget {
    pub fn from_descriptor(desc: &ProcessDescriptor, context: &CpuContext) -> Result<Self, DispatchError> {
        if desc.state == ProcessState::Exited || desc.state == ProcessState::Blocked {
            return Err(DispatchError::NotRunnable);
        }
        let frame = SwitchFrame::from_context(context);
        if !frame.is_well_formed() { return Err(DispatchError::InvalidProcess); }
        Ok(Self { process: desc.id, frame })
    }
}

/// Scheduler-side gate: validate both execution contexts before the backend.
pub fn prepare_dispatch(current: &SwitchFrame, target: &DispatchTarget) -> Result<(), DispatchError> {
    validate_backend_frames(current, &target.frame).map_err(DispatchError::Backend)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn descriptor(state: ProcessState) -> ProcessDescriptor {
        ProcessDescriptor { id: ProcessId(7), state, budget: super::super::ResourceBudget::unlimited() }
    }
    fn context(rip: u64, rsp: u64) -> CpuContext { CpuContext::kernel_entry(rip, rsp, 0x1000) }
    #[test]
    fn runnable_process_becomes_dispatch_target() {
        let target = DispatchTarget::from_descriptor(&descriptor(ProcessState::Runnable), &context(0x500000, 0x900000)).unwrap();
        assert_eq!(target.process, ProcessId(7));
        assert_eq!(target.frame.rip, 0x500000);
    }
    #[test]
    fn blocked_process_cannot_be_dispatched() {
        assert_eq!(DispatchTarget::from_descriptor(&descriptor(ProcessState::Blocked), &context(0x500000, 0x900000)), Err(DispatchError::NotRunnable));
    }
    #[test]
    fn dispatcher_rejects_invalid_current_context() {
        let target = DispatchTarget::from_descriptor(&descriptor(ProcessState::Runnable), &context(0x500000, 0x900000)).unwrap();
        assert!(matches!(prepare_dispatch(&SwitchFrame::default(), &target), Err(DispatchError::Backend(_))));
    }
}
