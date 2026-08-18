#![no_std]

pub mod context;
pub mod context_switch;
pub mod context_switch_abi;
pub mod dispatch;
pub mod dispatch_runtime;
pub mod scheduler;
pub mod x86_64_backend;

use core::sync::atomic::{AtomicU64, Ordering};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProcessId(pub u64);

pub struct ProcessTable { next_id: AtomicU64 }
impl Default for ProcessTable { fn default() -> Self { Self::new() } }
impl ProcessTable {
    pub const fn new() -> Self { Self { next_id: AtomicU64::new(1) } }
    pub fn allocate_id(&self) -> ProcessId { ProcessId(self.next_id.fetch_add(1, Ordering::Relaxed)) }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessState { Created = 0, Runnable = 1, Running = 2, Blocked = 3, Exited = 4 }

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceBudget { pub cpu_ticks: u64, pub memory_bytes: u64, pub ipc_messages: u64 }
impl ResourceBudget {
    pub const fn unlimited() -> Self { Self { cpu_ticks: u64::MAX, memory_bytes: u64::MAX, ipc_messages: u64::MAX } }
    pub const fn permits_cpu(&self, ticks: u64) -> bool { ticks <= self.cpu_ticks }
    pub const fn permits_memory(&self, bytes: u64) -> bool { bytes <= self.memory_bytes }
    pub const fn permits_ipc(&self, messages: u64) -> bool { messages <= self.ipc_messages }
    pub fn consume_cpu(&mut self, ticks: u64) -> bool { if ticks > self.cpu_ticks { return false; } self.cpu_ticks -= ticks; true }
    pub fn consume_memory(&mut self, bytes: u64) -> bool { if bytes > self.memory_bytes { return false; } self.memory_bytes -= bytes; true }
    pub fn consume_ipc(&mut self, messages: u64) -> bool { if messages > self.ipc_messages { return false; } self.ipc_messages -= messages; true }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessDescriptor { pub id: ProcessId, pub state: ProcessState, pub budget: ResourceBudget }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn process_ids_are_monotonic() {
        let table = ProcessTable::new();
        assert_eq!(table.allocate_id(), ProcessId(1));
        assert_eq!(table.allocate_id(), ProcessId(2));
        assert_eq!(table.allocate_id(), ProcessId(3));
    }
    #[test]
    fn budget_consumption_is_atomic_on_failure() {
        let mut b = ResourceBudget { cpu_ticks: 10, memory_bytes: 4096, ipc_messages: 2 };
        assert!(b.consume_cpu(4));
        assert_eq!(b.cpu_ticks, 6);
        assert!(!b.consume_cpu(7));
        assert_eq!(b.cpu_ticks, 6);
        assert!(b.consume_memory(1024));
        assert_eq!(b.memory_bytes, 3072);
        assert!(!b.consume_memory(4096));
        assert_eq!(b.memory_bytes, 3072);
        assert!(b.consume_ipc(2));
        assert_eq!(b.ipc_messages, 0);
        assert!(!b.consume_ipc(1));
        assert_eq!(b.ipc_messages, 0);
    }
    #[test]
    fn budget_permission_checks_are_non_mutating() {
        let b = ResourceBudget { cpu_ticks: 10, memory_bytes: 4096, ipc_messages: 2 };
        assert!(b.permits_cpu(10));
        assert!(!b.permits_cpu(11));
        assert!(b.permits_memory(4096));
        assert!(!b.permits_memory(4097));
        assert!(b.permits_ipc(2));
        assert!(!b.permits_ipc(3));
    }
}
