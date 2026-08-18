#![no_std]

use crate::process::{ProcessDescriptor, ProcessId, ProcessState, ResourceBudget};
use crate::scheduler::{DispatchAction, Scheduler};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoreError {
    ProcessTableFull,
    InvalidTransition,
    BudgetExceeded,
}

/// A bounded integration boundary for the A-C execution core.
/// It keeps scheduling, lifecycle and resource accounting in one deterministic
/// state machine without allocating or depending on hardware drivers.
pub struct ExecutionCore<const N: usize> {
    scheduler: Scheduler<N>,
    processes: [Option<ProcessDescriptor>; N],
    next_id: u64,
}

impl<const N: usize> Default for ExecutionCore<N> {
    fn default() -> Self { Self::new() }
}

impl<const N: usize> ExecutionCore<N> {
    pub const fn new() -> Self {
        Self { scheduler: Scheduler::new(), processes: [None; N], next_id: 1 }
    }

    pub fn create_process(&mut self, budget: ResourceBudget) -> Result<ProcessId, CoreError> {
        let mut slot = 0;
        while slot < N {
            if self.processes[slot].is_none() {
                let id = ProcessId(self.next_id);
                self.next_id = self.next_id.saturating_add(1);
                let mut descriptor = ProcessDescriptor { id, state: ProcessState::Created, budget };
                descriptor.transition(ProcessState::Runnable).map_err(|_| CoreError::InvalidTransition)?;
                self.processes[slot] = Some(descriptor);
                if !self.scheduler.enqueue(id) {
                    self.processes[slot] = None;
                    return Err(CoreError::ProcessTableFull);
                }
                return Ok(id);
            }
            slot += 1;
        }
        Err(CoreError::ProcessTableFull)
    }

    pub fn schedule(&mut self) -> DispatchAction {
        self.scheduler.request_reschedule();
        let action = self.scheduler.schedule_boundary();
        if let DispatchAction::SwitchTo(id) = action {
            self.set_running(id);
        }
        action
    }

    pub fn consume_cpu(&mut self, id: ProcessId, ticks: u64) -> Result<(), CoreError> {
        match self.find_mut(id) {
            Some(process) if process.budget.consume_cpu(ticks) => Ok(()),
            Some(_) => Err(CoreError::BudgetExceeded),
            None => Err(CoreError::InvalidTransition),
        }
    }

    pub fn exit(&mut self, id: ProcessId) -> Result<(), CoreError> {
        let process = self.find_mut(id).ok_or(CoreError::InvalidTransition)?;
        process.transition(ProcessState::Exited).map_err(|_| CoreError::InvalidTransition)
    }

    pub const fn current(&self) -> Option<ProcessId> { self.scheduler.current() }

    fn set_running(&mut self, id: ProcessId) {
        let mut i = 0;
        while i < N {
            if let Some(process) = self.processes[i].as_mut() {
                if process.id == id {
                    let _ = process.transition(ProcessState::Running);
                } else if process.state == ProcessState::Running {
                    let _ = process.transition(ProcessState::Runnable);
                }
            }
            i += 1;
        }
    }

    fn find_mut(&mut self, id: ProcessId) -> Option<&mut ProcessDescriptor> {
        let mut i = 0;
        while i < N {
            if self.processes[i].as_ref().map(|p| p.id) == Some(id) {
                return self.processes[i].as_mut();
            }
            i += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> ResourceBudget {
        ResourceBudget { cpu_ticks: 4, memory_bytes: 4096, ipc_messages: 2 }
    }

    #[test]
    fn lifecycle_and_preemption_are_bounded() {
        let mut core: ExecutionCore<2> = ExecutionCore::new();
        let first = core.create_process(budget()).unwrap();
        let second = core.create_process(budget()).unwrap();
        assert_eq!(core.schedule(), DispatchAction::SwitchTo(first));
        assert_eq!(core.current(), Some(first));
        assert_eq!(core.schedule(), DispatchAction::SwitchTo(second));
        assert_eq!(core.current(), Some(second));
    }

    #[test]
    fn capacity_and_budget_fail_closed() {
        let mut core: ExecutionCore<1> = ExecutionCore::new();
        let id = core.create_process(budget()).unwrap();
        assert_eq!(core.create_process(budget()), Err(CoreError::ProcessTableFull));
        assert!(core.consume_cpu(id, 4).is_ok());
        assert_eq!(core.consume_cpu(id, 1), Err(CoreError::BudgetExceeded));
    }

    #[test]
    fn invalid_exit_is_rejected() {
        let mut core: ExecutionCore<1> = ExecutionCore::new();
        assert_eq!(core.exit(ProcessId(99)), Err(CoreError::InvalidTransition));
    }
}
