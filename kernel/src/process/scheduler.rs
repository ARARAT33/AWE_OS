#![no_std]

use super::{ProcessContext, ProcessDescriptor, ProcessId, ProcessState};
use super::context_switch::SwitchFrame;
use super::dispatch::{prepare_dispatch, DispatchError, DispatchTarget};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SchedulerError { QueueFull, Empty, NotRunnable, UnknownProcess, InvalidContext, Dispatch(DispatchError) }

/// Deterministic bounded round-robin scheduler core.
pub struct Scheduler<const N: usize> {
    queue: [Option<ProcessId>; N],
    head: usize,
    len: usize,
    ticks: u64,
}

impl<const N: usize> Scheduler<N> {
    pub const fn new() -> Self { Self { queue: [None; N], head: 0, len: 0, ticks: 0 } }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
    pub const fn len(&self) -> usize { self.len }
    pub const fn ticks(&self) -> u64 { self.ticks }

    pub fn enqueue(&mut self, process: &ProcessDescriptor) -> Result<(), SchedulerError> {
        if !matches!(process.state, ProcessState::Runnable | ProcessState::Running) { return Err(SchedulerError::NotRunnable); }
        if self.len == N { return Err(SchedulerError::QueueFull); }
        let index = (self.head + self.len) % N;
        self.queue[index] = Some(process.id);
        self.len += 1;
        Ok(())
    }

    pub fn dequeue(&mut self) -> Result<ProcessId, SchedulerError> {
        if self.len == 0 { return Err(SchedulerError::Empty); }
        let id = self.queue[self.head].take().ok_or(SchedulerError::Empty)?;
        self.head = (self.head + 1) % N;
        self.len -= 1;
        self.ticks = self.ticks.wrapping_add(1);
        Ok(id)
    }

    pub fn yield_process(&mut self, process: &ProcessDescriptor) -> Result<(), SchedulerError> { self.enqueue(process) }

    /// Selects the next queued process and binds its ID to an owned CPU context.
    pub fn prepare_next(&mut self, contexts: &[ProcessContext; N]) -> Result<DispatchTarget, SchedulerError> {
        let id = self.dequeue()?;
        let context = contexts.iter().find(|ctx| ctx.process_id == id).ok_or(SchedulerError::UnknownProcess)?;
        if !context.is_valid() { return Err(SchedulerError::InvalidContext); }
        let descriptor = ProcessDescriptor { id, state: ProcessState::Runnable, budget: super::ResourceBudget::unlimited() };
        DispatchTarget::from_descriptor(&descriptor, &context.cpu).map_err(SchedulerError::Dispatch)
    }

    /// Validates the complete scheduler-to-dispatch boundary without changing CPU state.
    pub fn prepare_switch(&mut self, current: &SwitchFrame, contexts: &[ProcessContext; N]) -> Result<DispatchTarget, SchedulerError> {
        let target = self.prepare_next(contexts)?;
        prepare_dispatch(current, &target).map_err(SchedulerError::Dispatch)?;
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn process(id: u64, state: ProcessState) -> ProcessDescriptor { ProcessDescriptor { id: ProcessId(id), state, budget: super::super::ResourceBudget::unlimited() } }
    fn context(id: u64, rip: u64, rsp: u64) -> ProcessContext { ProcessContext::new(ProcessId(id), super::super::context::CpuContext::kernel_entry(rip, rsp, 0x100000)) }

    #[test]
    fn round_robin_order_is_deterministic() {
        let mut s = Scheduler::<4>::new();
        s.enqueue(&process(1, ProcessState::Runnable)).unwrap(); s.enqueue(&process(2, ProcessState::Runnable)).unwrap(); s.enqueue(&process(3, ProcessState::Runnable)).unwrap();
        assert_eq!(s.dequeue().unwrap(), ProcessId(1)); assert_eq!(s.dequeue().unwrap(), ProcessId(2)); assert_eq!(s.dequeue().unwrap(), ProcessId(3)); assert_eq!(s.ticks(), 3);
    }
    #[test]
    fn blocked_process_never_enters_queue() { let mut s=Scheduler::<2>::new(); assert_eq!(s.enqueue(&process(1,ProcessState::Blocked)),Err(SchedulerError::NotRunnable)); assert!(s.is_empty()); }
    #[test]
    fn queue_capacity_is_enforced() { let mut s=Scheduler::<1>::new(); s.enqueue(&process(1,ProcessState::Runnable)).unwrap(); assert_eq!(s.enqueue(&process(2,ProcessState::Runnable)),Err(SchedulerError::QueueFull)); }
    #[test]
    fn queued_process_is_bound_to_its_cpu_context() {
        let mut s=Scheduler::<2>::new(); s.enqueue(&process(7,ProcessState::Runnable)).unwrap();
        let contexts=[context(7,0x500000,0x900000),context(9,0x600000,0xa00000)];
        let target=s.prepare_next(&contexts).unwrap();
        assert_eq!(target.process,ProcessId(7)); assert_eq!(target.frame.rip,0x500000); assert_eq!(target.frame.rsp,0x900000);
    }
    #[test]
    fn invalid_context_never_reaches_dispatch() {
        let mut s=Scheduler::<1>::new(); s.enqueue(&process(7,ProcessState::Runnable)).unwrap();
        let contexts=[context(7,0,0x900000)];
        assert_eq!(s.prepare_next(&contexts),Err(SchedulerError::InvalidContext));
    }
}
