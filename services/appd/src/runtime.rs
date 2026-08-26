#![no_std]

use super::{AppId, AppManifest, AppState, validate_manifest, AWE_APP_ABI_MAJOR, AWE_APP_ABI_MINOR};

pub const MAX_APPS: usize = 32;
pub const MAX_FAILURES: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeError { Full, InvalidManifest, Duplicate, NotFound, CapabilityDenied, SpawnFailed, InvalidTransition, Quarantined }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeApp { pub manifest: AppManifest, pub state: AppState, pub process_id: u64, pub window_id: u16, pub failures: u8 }

pub type SpawnFn = fn(app: AppId, memory_pages: u32, capability_mask: u64) -> Result<u64, ()>;
pub type WindowFn = fn(app: AppId) -> Result<u16, ()>;

pub struct Supervisor { apps: [Option<RuntimeApp>; MAX_APPS], count: usize, spawn: SpawnFn, create_window: WindowFn }
impl Supervisor {
    pub const fn new(spawn: SpawnFn, create_window: WindowFn) -> Self { Self { apps: [None; MAX_APPS], count: 0, spawn, create_window } }
    pub const fn len(&self) -> usize { self.count }
    pub fn admit(&mut self, manifest: AppManifest) -> Result<(), RuntimeError> {
        validate_manifest(manifest).map_err(|_| RuntimeError::InvalidManifest)?;
        if self.find(manifest.id).is_some() { return Err(RuntimeError::Duplicate); }
        let slot = self.apps.iter().position(Option::is_none).ok_or(RuntimeError::Full)?;
        self.apps[slot] = Some(RuntimeApp { manifest, state: AppState::Installed, process_id: 0, window_id: 0, failures: 0 });
        self.count += 1; Ok(())
    }
    pub fn start(&mut self, id: AppId, caller_capabilities: u64) -> Result<u64, RuntimeError> {
        let index = self.find(id).ok_or(RuntimeError::NotFound)?;
        let record = self.apps[index].unwrap();
        if record.state == AppState::Quarantined { return Err(RuntimeError::Quarantined); }
        if record.manifest.capability_mask & !caller_capabilities != 0 { return Err(RuntimeError::CapabilityDenied); }
        if !matches!(record.state, AppState::Installed | AppState::Stopped | AppState::Failed) { return Err(RuntimeError::InvalidTransition); }
        self.apps[index].as_mut().unwrap().state = AppState::Starting;
        let pid = match (self.spawn)(id, record.manifest.memory_limit_pages, record.manifest.capability_mask) {
            Ok(pid) if pid != 0 => pid,
            _ => { self.apps[index].as_mut().unwrap().state = AppState::Failed; return Err(RuntimeError::SpawnFailed); }
        };
        let window = match (self.create_window)(id) {
            Ok(window) if window != 0 => window,
            _ => { self.apps[index].as_mut().unwrap().state = AppState::Failed; return Err(RuntimeError::SpawnFailed); }
        };
        let record = self.apps[index].as_mut().unwrap();
        record.process_id = pid; record.window_id = window; record.failures = 0; record.state = AppState::Running; Ok(pid)
    }
    pub fn stop(&mut self, id: AppId) -> Result<(), RuntimeError> {
        let index = self.find(id).ok_or(RuntimeError::NotFound)?;
        let record = self.apps[index].as_mut().unwrap();
        if record.state != AppState::Running { return Err(RuntimeError::InvalidTransition); }
        record.state = AppState::Stopped; record.process_id = 0; Ok(())
    }
    pub fn report_failure(&mut self, id: AppId) -> Result<AppState, RuntimeError> {
        let index = self.find(id).ok_or(RuntimeError::NotFound)?;
        let record = self.apps[index].as_mut().unwrap();
        if record.state != AppState::Running { return Err(RuntimeError::InvalidTransition); }
        record.failures = record.failures.saturating_add(1);
        record.state = if record.failures > MAX_FAILURES { AppState::Quarantined } else { AppState::Failed };
        Ok(record.state)
    }
    pub fn state(&self, id: AppId) -> Option<AppState> { self.find(id).and_then(|i| self.apps[i].map(|r| r.state)) }
    pub fn record(&self, id: AppId) -> Option<RuntimeApp> { self.find(id).and_then(|i| self.apps[i]) }
    fn find(&self, id: AppId) -> Option<usize> { self.apps.iter().position(|r| r.map(|a| a.manifest.id) == Some(id)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spawn(app: AppId, memory: u32, _caps: u64) -> Result<u64, ()> { if app.0 == 0 || memory == 0 { Err(()) } else { Ok(0x1000 + app.0) } }
    fn window(app: AppId) -> Result<u16, ()> { if app.0 == 0 || app.0 > u16::MAX as u64 { Err(()) } else { Ok(app.0 as u16) } }
    fn manifest(id: u64) -> AppManifest { AppManifest { id: AppId(id), abi_major: AWE_APP_ABI_MAJOR, abi_minor: AWE_APP_ABI_MINOR, memory_limit_pages: 4, capability_mask: 0b101, dependency_count: 0, resource_count: 0 } }
    #[test] fn admission_start_and_stop_are_real_lifecycle_steps() { let mut s = Supervisor::new(spawn, window); s.admit(manifest(1)).unwrap(); let pid = s.start(AppId(1), 0b111).unwrap(); assert_eq!(pid, 0x1001); assert_eq!(s.record(AppId(1)).unwrap().window_id, 1); assert_eq!(s.state(AppId(1)), Some(AppState::Running)); s.stop(AppId(1)).unwrap(); assert_eq!(s.state(AppId(1)), Some(AppState::Stopped)); }
    #[test] fn capability_admission_is_fail_closed() { let mut s = Supervisor::new(spawn, window); s.admit(manifest(2)).unwrap(); assert_eq!(s.start(AppId(2), 0b001), Err(RuntimeError::CapabilityDenied)); }
    #[test] fn repeated_failures_quarantine_app() { let mut s = Supervisor::new(spawn, window); s.admit(manifest(3)).unwrap(); s.start(AppId(3), 0b111).unwrap(); for _ in 0..MAX_FAILURES { assert_eq!(s.report_failure(AppId(3)).unwrap(), AppState::Failed); s.start(AppId(3), 0b111).unwrap(); } assert_eq!(s.report_failure(AppId(3)).unwrap(), AppState::Quarantined); }
}
