#![no_std]

pub const ABI_MAJOR: u16 = 1;
pub const MAX_DEVICES: usize = 32;
pub const MAX_SERVICES: usize = 32;
pub const MAX_LOGS: usize = 128;
pub const MAX_WINDOWS: usize = 16;
pub const MAX_PATH: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Capacity,
    Invalid,
    Conflict,
    Permission,
    State,
    Bounds,
    NotFound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootInfo {
    pub abi_major: u16,
    pub abi_minor: u16,
    pub memory_bytes: u64,
    pub cpu_count: u16,
}
impl BootInfo {
    pub const fn validate(self) -> Result<(), Error> {
        if self.abi_major != ABI_MAJOR || self.memory_bytes < 1024 * 1024 || self.cpu_count == 0 {
            Err(Error::Invalid)
        } else { Ok(()) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceState { Discovered, Probed, Bound, Running, Suspended, Stopped, Removed, Quarantined }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Device { pub id: u16, pub class: u16, pub state: DeviceState, pub owner: u16 }

pub struct DeviceManager { items: [Option<Device>; MAX_DEVICES] }
impl DeviceManager {
    pub const fn new() -> Self { Self { items: [None; MAX_DEVICES] } }
    pub fn discover(&mut self, id: u16, class: u16) -> Result<(), Error> {
        if self.items.iter().flatten().any(|d| d.id == id) { return Err(Error::Conflict); }
        let slot = self.items.iter().position(Option::is_none).ok_or(Error::Capacity)?;
        self.items[slot] = Some(Device { id, class, state: DeviceState::Discovered, owner: 0 }); Ok(())
    }
    pub fn transition(&mut self, id: u16, next: DeviceState, owner: u16) -> Result<(), Error> {
        let d = self.items.iter_mut().flatten().find(|d| d.id == id).ok_or(Error::NotFound)?;
        let valid = matches!((d.state, next),
            (DeviceState::Discovered, DeviceState::Probed) |
            (DeviceState::Probed, DeviceState::Bound) |
            (DeviceState::Bound, DeviceState::Running) |
            (DeviceState::Running, DeviceState::Suspended) |
            (DeviceState::Suspended, DeviceState::Running) |
            (DeviceState::Running, DeviceState::Stopped) |
            (DeviceState::Stopped, DeviceState::Removed) |
            (_, DeviceState::Quarantined));
        if !valid { return Err(Error::State); }
        d.owner = owner; d.state = next; Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceState { Registered, Starting, Running, Failed, Quarantined, Stopped }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Service { pub id: u16, pub state: ServiceState, pub caps: u64, pub memory_pages: u32 }
pub struct ServiceManager { items: [Option<Service>; MAX_SERVICES] }
impl ServiceManager {
    pub const fn new() -> Self { Self { items: [None; MAX_SERVICES] } }
    pub fn register(&mut self, id: u16, caps: u64, memory_pages: u32) -> Result<(), Error> {
        if memory_pages == 0 || self.items.iter().flatten().any(|s| s.id == id) { return Err(Error::Invalid); }
        let slot = self.items.iter().position(Option::is_none).ok_or(Error::Capacity)?;
        self.items[slot] = Some(Service { id, state: ServiceState::Registered, caps, memory_pages }); Ok(())
    }
    pub fn start(&mut self, id: u16) -> Result<(), Error> {
        let s = self.items.iter_mut().flatten().find(|s| s.id == id).ok_or(Error::NotFound)?;
        if s.state != ServiceState::Registered { return Err(Error::State); }
        s.state = ServiceState::Starting; s.state = ServiceState::Running; Ok(())
    }
    pub fn fail_and_quarantine(&mut self, id: u16) -> Result<(), Error> {
        let s = self.items.iter_mut().flatten().find(|s| s.id == id).ok_or(Error::NotFound)?;
        if s.state != ServiceState::Running { return Err(Error::State); }
        s.state = ServiceState::Failed; s.state = ServiceState::Quarantined; Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot { A, B }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotState { Active, Staged, Booting, Healthy, Failed, Quarantined }
pub struct Recovery { active: Slot, generation: u64, staged: Option<(Slot, u64)>, state: SlotState }
impl Recovery {
    pub const fn new(generation: u64) -> Self { Self { active: Slot::A, generation, staged: None, state: SlotState::Active } }
    pub fn stage(&mut self, slot: Slot, generation: u64) -> Result<(), Error> {
        if generation <= self.generation || slot == self.active { return Err(Error::Conflict); }
        self.staged = Some((slot, generation)); self.state = SlotState::Staged; Ok(())
    }
    pub fn boot(&mut self) -> Result<Slot, Error> {
        let (slot, _) = self.staged.ok_or(Error::State)?; self.state = SlotState::Booting; Ok(slot)
    }
    pub fn healthy(&mut self) -> Result<(), Error> {
        let (slot, generation) = self.staged.take().ok_or(Error::State)?;
        if self.state != SlotState::Booting { return Err(Error::State); }
        self.active = slot; self.generation = generation; self.state = SlotState::Healthy; Ok(())
    }
    pub fn fail_and_rollback(&mut self) -> Result<Slot, Error> {
        if self.state != SlotState::Booting { return Err(Error::State); }
        self.staged = None; self.state = SlotState::Failed; Ok(self.active)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogRecord { pub seq: u64, pub code: u16, pub arg: u64 }
pub struct Logger { records: [Option<LogRecord>; MAX_LOGS], next: u64 }
impl Logger {
    pub const fn new() -> Self { Self { records: [None; MAX_LOGS], next: 0 } }
    pub fn push(&mut self, code: u16, arg: u64) { let i = (self.next as usize) % MAX_LOGS; self.records[i] = Some(LogRecord { seq: self.next, code, arg }); self.next = self.next.saturating_add(1); }
    pub const fn len(&self) -> usize { if self.next < MAX_LOGS as u64 { self.next as usize } else { MAX_LOGS } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window { pub id: u16, pub x: i16, pub y: i16, pub w: u16, pub h: u16 }
pub struct Compositor { windows: [Option<Window>; MAX_WINDOWS], count: usize }
impl Compositor {
    pub const fn new() -> Self { Self { windows: [None; MAX_WINDOWS], count: 0 } }
    pub fn create(&mut self, window: Window) -> Result<(), Error> {
        if window.w == 0 || window.h == 0 || self.count == MAX_WINDOWS || self.windows.iter().flatten().any(|w| w.id == window.id) { return Err(Error::Invalid); }
        let i = self.windows.iter().position(Option::is_none).ok_or(Error::Capacity)?; self.windows[i] = Some(window); self.count += 1; Ok(())
    }
    pub const fn count(&self) -> usize { self.count }
}

pub fn validate_path(path: &[u8]) -> Result<(), Error> {
    if path.is_empty() || path.len() > MAX_PATH || path[0] != b'/' || path.windows(2).any(|w| w == b"..") { Err(Error::Bounds) } else { Ok(()) }
}

pub fn checksum32(data: &[u8]) -> u32 {
    let mut h = 0x811c9dc5u32;
    let mut i = 0; while i < data.len() { h ^= data[i] as u32; h = h.wrapping_mul(0x01000193); i += 1; }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn boot_is_fail_closed() { assert!(BootInfo { abi_major: 1, abi_minor: 0, memory_bytes: 16*1024*1024, cpu_count: 2 }.validate().is_ok()); assert!(BootInfo { abi_major: 2, abi_minor: 0, memory_bytes: 16*1024*1024, cpu_count: 2 }.validate().is_err()); }
    #[test] fn device_lifecycle_is_bounded() { let mut m=DeviceManager::new(); m.discover(1,2).unwrap(); m.transition(1,DeviceState::Probed,7).unwrap(); m.transition(1,DeviceState::Bound,7).unwrap(); m.transition(1,DeviceState::Running,7).unwrap(); assert!(m.transition(1,DeviceState::Removed,7).is_err()); m.transition(1,DeviceState::Quarantined,7).unwrap(); }
    #[test] fn service_recovery_and_logging_work() { let mut s=ServiceManager::new(); s.register(1,3,64).unwrap(); s.start(1).unwrap(); s.fail_and_quarantine(1).unwrap(); let mut r=Recovery::new(4); r.stage(Slot::B,5).unwrap(); assert_eq!(r.boot().unwrap(),Slot::B); r.healthy().unwrap(); let mut l=Logger::new(); for i in 0..140 { l.push(i, i as u64); } assert_eq!(l.len(),128); }
    #[test] fn ui_and_input_boundaries_are_safe() { let mut c=Compositor::new(); c.create(Window{id:1,x:0,y:0,w:800,h:600}).unwrap(); assert_eq!(c.count(),1); assert!(validate_path(b"/etc/awe.conf").is_ok()); assert!(validate_path(b"/etc/../secret").is_err()); }
    #[test] fn deterministic_checksum_is_stable() { assert_eq!(checksum32(b"AWE_OS"), checksum32(b"AWE_OS")); assert_ne!(checksum32(b"AWE_OS"), checksum32(b"AWE_X")); }
}
