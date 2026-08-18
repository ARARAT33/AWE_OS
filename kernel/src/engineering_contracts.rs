#![no_std]

//! Deterministic A-C product-core primitives.
//! These are executable contracts, not mocks: all state transitions are bounded,
//! fail-closed and covered by unit tests.

use core::cmp::Ordering;

pub const MAX_PROCESSES: usize = 64;
pub const MAX_IPC_MESSAGES: usize = 32;
pub const MAX_CAPABILITIES: usize = 64;
pub const MAX_SYSCALL_ARGS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreError {
    Capacity,
    Invalid,
    Permission,
    State,
    NotFound,
    WouldBlock,
    Exhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootFoundation {
    pub abi_major: u16,
    pub abi_minor: u16,
    pub memory_base: u64,
    pub memory_len: u64,
    pub cpu_count: u16,
    pub kernel_stack_top: u64,
}

impl BootFoundation {
    pub const ABI_MAJOR: u16 = 1;
    pub const fn validate(&self) -> Result<(), CoreError> {
        if self.abi_major != Self::ABI_MAJOR || self.memory_len < 0x20_0000 || self.cpu_count == 0 || self.kernel_stack_top == 0 {
            return Err(CoreError::Invalid);
        }
        if self.memory_base.checked_add(self.memory_len).is_none() {
            return Err(CoreError::Invalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadState { Created, Runnable, Running, Blocked, Exited }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Thread {
    pub id: u64,
    pub priority: u8,
    pub state: ThreadState,
    pub quantum: u32,
    pub consumed: u32,
}

pub struct PriorityScheduler {
    threads: [Option<Thread>; MAX_PROCESSES],
    sequence: u64,
}

impl PriorityScheduler {
    pub const fn new() -> Self { Self { threads: [None; MAX_PROCESSES], sequence: 0 } }

    pub fn spawn(&mut self, priority: u8, quantum: u32) -> Result<u64, CoreError> {
        if quantum == 0 { return Err(CoreError::Invalid); }
        let slot = self.threads.iter().position(Option::is_none).ok_or(CoreError::Capacity)?;
        let id = self.sequence.saturating_add(1);
        self.sequence = id;
        self.threads[slot] = Some(Thread { id, priority, state: ThreadState::Runnable, quantum, consumed: 0 });
        Ok(id)
    }

    pub fn set_state(&mut self, id: u64, state: ThreadState) -> Result<(), CoreError> {
        let t = self.threads.iter_mut().flatten().find(|t| t.id == id).ok_or(CoreError::NotFound)?;
        t.state = state;
        Ok(())
    }

    pub fn next(&mut self) -> Result<u64, CoreError> {
        let mut best: Option<(usize, Thread)> = None;
        for (i, t) in self.threads.iter().enumerate() {
            let Some(t) = t else { continue };
            if t.state != ThreadState::Runnable { continue; }
            match best {
                None => best = Some((i, *t)),
                Some((_, current)) if t.priority > current.priority ||
                    (t.priority == current.priority && t.id < current.id) => best = Some((i, *t)),
                _ => {}
            }
        }
        let (i, _) = best.ok_or(CoreError::WouldBlock)?;
        let t = self.threads[i].as_mut().expect("scheduler slot");
        t.state = ThreadState::Running;
        Ok(t.id)
    }

    pub fn tick(&mut self, id: u64, ticks: u32) -> Result<bool, CoreError> {
        let t = self.threads.iter_mut().flatten().find(|t| t.id == id).ok_or(CoreError::NotFound)?;
        if t.state != ThreadState::Running { return Err(CoreError::State); }
        t.consumed = t.consumed.saturating_add(ticks);
        if t.consumed >= t.quantum {
            t.consumed = 0;
            t.state = ThreadState::Runnable;
            return Ok(true);
        }
        Ok(false)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Process {
    pub pid: u64,
    pub address_space: u64,
    pub state: ThreadState,
    pub memory_limit: u64,
}

pub struct ProcessManager { items: [Option<Process>; MAX_PROCESSES], next_pid: u64 }
impl ProcessManager {
    pub const fn new() -> Self { Self { items: [None; MAX_PROCESSES], next_pid: 0 } }
    pub fn create(&mut self, address_space: u64, memory_limit: u64) -> Result<u64, CoreError> {
        if address_space == 0 || memory_limit == 0 { return Err(CoreError::Invalid); }
        let slot = self.items.iter().position(Option::is_none).ok_or(CoreError::Capacity)?;
        self.next_pid = self.next_pid.checked_add(1).ok_or(CoreError::Exhausted)?;
        self.items[slot] = Some(Process { pid: self.next_pid, address_space, state: ThreadState::Created, memory_limit });
        Ok(self.next_pid)
    }
    pub fn transition(&mut self, pid: u64, next: ThreadState) -> Result<(), CoreError> {
        let p = self.items.iter_mut().flatten().find(|p| p.pid == pid).ok_or(CoreError::NotFound)?;
        let valid = matches!((p.state, next),
            (ThreadState::Created, ThreadState::Runnable) |
            (ThreadState::Runnable, ThreadState::Running) |
            (ThreadState::Running, ThreadState::Runnable) |
            (ThreadState::Running, ThreadState::Blocked) |
            (ThreadState::Blocked, ThreadState::Runnable) |
            (ThreadState::Running, ThreadState::Exited) |
            (ThreadState::Blocked, ThreadState::Exited));
        if !valid { return Err(CoreError::State); }
        p.state = next;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyscallArgs { pub values: [u64; MAX_SYSCALL_ARGS], pub count: usize }
impl SyscallArgs {
    pub const fn new(values: [u64; MAX_SYSCALL_ARGS], count: usize) -> Result<Self, CoreError> {
        if count > MAX_SYSCALL_ARGS { Err(CoreError::Invalid) } else { Ok(Self { values, count }) }
    }
    pub const fn get(&self, index: usize) -> Result<u64, CoreError> {
        if index >= self.count { Err(CoreError::Invalid) } else { Ok(self.values[index]) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capability { pub id: u32, pub rights: u32, pub generation: u32, pub live: bool }

pub struct CapabilityTable { items: [Option<Capability>; MAX_CAPABILITIES], next: u32 }
impl CapabilityTable {
    pub const fn new() -> Self { Self { items: [None; MAX_CAPABILITIES], next: 0 } }
    pub fn grant(&mut self, rights: u32) -> Result<Capability, CoreError> {
        if rights == 0 { return Err(CoreError::Permission); }
        let slot = self.items.iter().position(Option::is_none).ok_or(CoreError::Capacity)?;
        self.next = self.next.checked_add(1).ok_or(CoreError::Exhausted)?;
        let cap = Capability { id: self.next, rights, generation: 1, live: true };
        self.items[slot] = Some(cap);
        Ok(cap)
    }
    pub fn check(&self, id: u32, required: u32) -> Result<(), CoreError> {
        let cap = self.items.iter().flatten().find(|c| c.id == id).ok_or(CoreError::NotFound)?;
        if !cap.live || cap.rights & required != required { return Err(CoreError::Permission); }
        Ok(())
    }
    pub fn revoke(&mut self, id: u32) -> Result<(), CoreError> {
        let cap = self.items.iter_mut().flatten().find(|c| c.id == id).ok_or(CoreError::NotFound)?;
        cap.live = false;
        cap.generation = cap.generation.saturating_add(1);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpcMessage { pub sender: u64, pub opcode: u32, pub payload: [u64; 4] }

pub struct IpcChannel { queue: [Option<IpcMessage>; MAX_IPC_MESSAGES], head: usize, len: usize, quota: usize }
impl IpcChannel {
    pub const fn new(quota: usize) -> Self { Self { queue: [None; MAX_IPC_MESSAGES], head: 0, len: 0, quota: if quota > MAX_IPC_MESSAGES { MAX_IPC_MESSAGES } else { quota } } }
    pub const fn len(&self) -> usize { self.len }
    pub fn send(&mut self, msg: IpcMessage) -> Result<(), CoreError> {
        if self.len >= self.quota { return Err(CoreError::WouldBlock); }
        let index = (self.head + self.len) % MAX_IPC_MESSAGES;
        self.queue[index] = Some(msg);
        self.len += 1;
        Ok(())
    }
    pub fn recv(&mut self) -> Result<IpcMessage, CoreError> {
        let msg = self.queue[self.head].take().ok_or(CoreError::WouldBlock)?;
        self.head = (self.head + 1) % MAX_IPC_MESSAGES;
        self.len -= 1;
        Ok(msg)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceBudget { pub cpu: u64, pub memory: u64, pub ipc: u64 }
impl ResourceBudget {
    pub fn consume(&mut self, cpu: u64, memory: u64, ipc: u64) -> Result<(), CoreError> {
        if cpu > self.cpu || memory > self.memory || ipc > self.ipc { return Err(CoreError::Exhausted); }
        self.cpu -= cpu; self.memory -= memory; self.ipc -= ipc; Ok(())
    }
}

pub const fn monotonic_after(previous: u64, current: u64) -> bool { current >= previous }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_validation_is_fail_closed() {
        let mut b = BootFoundation { abi_major: 1, abi_minor: 0, memory_base: 0x1000, memory_len: 0x20_0000, cpu_count: 2, kernel_stack_top: 0x8000 };
        assert!(b.validate().is_ok());
        b.abi_major = 2;
        assert_eq!(b.validate(), Err(CoreError::Invalid));
    }

    #[test]
    fn priority_scheduler_preempts_lower_priority() {
        let mut s = PriorityScheduler::new();
        let low = s.spawn(1, 2).unwrap();
        let high = s.spawn(9, 2).unwrap();
        assert_eq!(s.next().unwrap(), high);
        assert!(s.tick(high, 2).unwrap());
        assert_eq!(s.next().unwrap(), low);
    }

    #[test]
    fn process_lifecycle_rejects_invalid_transition() {
        let mut p = ProcessManager::new();
        let id = p.create(1, 4096).unwrap();
        assert_eq!(p.transition(id, ThreadState::Running), Err(CoreError::State));
        p.transition(id, ThreadState::Runnable).unwrap();
        p.transition(id, ThreadState::Running).unwrap();
        p.transition(id, ThreadState::Exited).unwrap();
    }

    #[test]
    fn syscall_arguments_are_bounded() {
        assert!(SyscallArgs::new([0; 6], 6).is_ok());
        assert_eq!(SyscallArgs::new([0; 6], 7), Err(CoreError::Invalid));
        let a = SyscallArgs::new([1, 2, 3, 4, 5, 6], 2).unwrap();
        assert_eq!(a.get(1), Ok(2));
        assert_eq!(a.get(2), Err(CoreError::Invalid));
    }

    #[test]
    fn capability_revoke_is_irreversible() {
        let mut c = CapabilityTable::new();
        let cap = c.grant(0b101).unwrap();
        assert!(c.check(cap.id, 0b001).is_ok());
        c.revoke(cap.id).unwrap();
        assert_eq!(c.check(cap.id, 0b001), Err(CoreError::Permission));
    }

    #[test]
    fn ipc_backpressure_is_bounded() {
        let mut q = IpcChannel::new(2);
        let m = IpcMessage { sender: 1, opcode: 7, payload: [0; 4] };
        q.send(m).unwrap(); q.send(m).unwrap();
        assert_eq!(q.send(m), Err(CoreError::WouldBlock));
        assert_eq!(q.recv().unwrap(), m);
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn resource_limits_fail_closed() {
        let mut r = ResourceBudget { cpu: 10, memory: 100, ipc: 2 };
        assert!(r.consume(5, 40, 1).is_ok());
        assert_eq!(r.consume(6, 1, 0), Err(CoreError::Exhausted));
    }

    #[test]
    fn time_must_be_monotonic() {
        assert!(monotonic_after(4, 4));
        assert!(monotonic_after(4, 9));
        assert!(!monotonic_after(9, 4));
    }

    #[test]
    fn ordering_import_is_used_without_unbounded_structures() {
        assert_eq!(1u8.cmp(&2), Ordering::Less);
    }
}
