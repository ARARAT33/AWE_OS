#![no_std]

use super::context_switch::SwitchFrame;
use super::dispatch::{prepare_dispatch, DispatchError, DispatchTarget};
use super::scheduler::{Scheduler, SchedulerError};
use super::ProcessContext;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuntimeDispatchError {
    Scheduler(SchedulerError),
    Dispatch(DispatchError),
}

/// Scheduler-to-architecture runtime boundary.
///
/// This layer deliberately performs no CPU mutation. It prepares and validates
/// the target frame first; the architecture backend is invoked by the caller
/// only after this function returns successfully.
pub fn prepare_runtime_switch<const N: usize>(
    scheduler: &mut Scheduler<N>,
    current: &SwitchFrame,
    contexts: &[ProcessContext; N],
) -> Result<DispatchTarget, RuntimeDispatchError> {
    let target = scheduler
        .prepare_next(contexts)
        .map_err(RuntimeDispatchError::Scheduler)?;
    prepare_dispatch(current, &target).map_err(RuntimeDispatchError::Dispatch)?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{ProcessDescriptor, ProcessId, ProcessState, ResourceBudget};
    use super::super::context::CpuContext;

    fn descriptor(id: u64) -> ProcessDescriptor {
        ProcessDescriptor { id: ProcessId(id), state: ProcessState::Runnable, budget: ResourceBudget::unlimited() }
    }

    fn context(id: u64, rip: u64, rsp: u64) -> ProcessContext {
        ProcessContext::new(ProcessId(id), CpuContext::kernel_entry(rip, rsp, 0x100000))
    }

    #[test]
    fn prepares_valid_runtime_target() {
        let mut scheduler = Scheduler::<2>::new();
        scheduler.enqueue(&descriptor(3)).unwrap();
        let contexts = [context(3, 0x500000, 0x900000), context(4, 0x600000, 0xa00000)];
        let current = SwitchFrame { rip: 0x400000, rsp: 0x800000, ..SwitchFrame::default() };
        let target = prepare_runtime_switch(&mut scheduler, &current, &contexts).unwrap();
        assert_eq!(target.process, ProcessId(3));
        assert_eq!(target.frame.rip, 0x500000);
        assert_eq!(target.frame.rsp, 0x900000);
    }

    #[test]
    fn invalid_current_is_stopped_before_backend() {
        let mut scheduler = Scheduler::<1>::new();
        scheduler.enqueue(&descriptor(3)).unwrap();
        let contexts = [context(3, 0x500000, 0x900000)];
        let current = SwitchFrame::default();
        assert!(matches!(
            prepare_runtime_switch(&mut scheduler, &current, &contexts),
            Err(RuntimeDispatchError::Dispatch(_))
        ));
    }
}
