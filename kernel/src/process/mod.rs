#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProcessId(pub u64);

pub struct ProcessTable {
    next_id: AtomicU64,
}

impl ProcessTable {
    pub const fn new() -> Self { Self { next_id: AtomicU64::new(1) } }

    /// Relaxed ordering is sufficient: IDs are unique counters, not a
    /// synchronization primitive for process state.
    pub fn allocate_id(&self) -> ProcessId {
        ProcessId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Created = 0,
    Runnable = 1,
    Running = 2,
    Blocked = 3,
    Exited = 4,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ResourceBudget {
    pub cpu_ticks: u64,
    pub memory_bytes: u64,
    pub ipc_messages: u64,
}

impl ResourceBudget {
    pub const fn unlimited() -> Self {
        Self { cpu_ticks: u64::MAX, memory_bytes: u64::MAX, ipc_messages: u64::MAX }
    }

    pub const fn permits_cpu(&self, ticks: u64) -> bool { ticks <= self.cpu_ticks }
    pub const fn permits_memory(&self, bytes: u64) -> bool { bytes <= self.memory_bytes }
    pub const fn permits_ipc(&self, messages: u64) -> bool { messages <= self.ipc_messages }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessDescriptor {
    pub id: ProcessId,
    pub state: ProcessState,
    pub budget: ResourceBudget,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_ids_are_unique() {
        let table = ProcessTable::new();
        assert_eq!(table.allocate_id().0, 1);
        assert_eq!(table.allocate_id().0, 2);
    }

    #[test]
    fn resource_budget_is_explicit() {
        let budget = ResourceBudget { cpu_ticks: 10, memory_bytes: 4096, ipc_messages: 3 };
        assert!(budget.permits_cpu(10));
        assert!(!budget.permits_cpu(11));
        assert!(budget.permits_memory(4096));
        assert!(!budget.permits_memory(4097));
        assert!(budget.permits_ipc(3));
        assert!(!budget.permits_ipc(4));
    }
}
