#![no_std]

use core::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use awe_appd::{AppId, AppManifest, AppState, AppSupervisor};
use awe_initd::{RestartPolicy, ServiceId, ServiceRuntimeSpec, ServiceSpec, ServiceState, Supervisor};
use crate::drivers::{KeyCode, Ps2Event};
use super::{CapabilitySet, EndUserRuntime, FramebufferInfo, InputEvent, RuntimeEvent};
pub use super::{RuntimeRect, WindowManager, WindowError};

static NEXT_PROCESS_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_WINDOW_ID: AtomicU16 = AtomicU16::new(1);

fn spawn_service(entry: usize, service: ServiceId, _memory_pages: u32, _cpu_budget: u32) -> Result<u64, ()> {
    if entry == 0 || service.0 == 0 { return Err(()); }
    Ok(NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed))
}
fn spawn_app(app: AppId, memory_pages: u32, _capabilities: u64) -> Result<u64, ()> {
    if app.0 == 0 || memory_pages == 0 { return Err(()); }
    Ok(NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed))
}
fn create_window(_app: AppId) -> Result<u16, ()> {
    let id = NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed);
    if id == 0 { Err(()) } else { Ok(id) }
}

const ALL: CapabilitySet = CapabilitySet(
    CapabilitySet::PROCESS.0 | CapabilitySet::MEMORY.0 | CapabilitySet::IPC.0 |
    CapabilitySet::DEVICE.0 | CapabilitySet::STORAGE.0 | CapabilitySet::NETWORK.0 | CapabilitySet::UI.0,
);

pub struct SystemRuntime {
    pub core: EndUserRuntime,
    pub windows: WindowManager,
    pub services: Supervisor,
    pub apps: AppSupervisor,
    pub cursor_x: i32,
    pub cursor_y: i32,
}

impl SystemRuntime {
    pub const fn new() -> Self {
        Self { core: EndUserRuntime::new(), windows: WindowManager::new(), services: Supervisor::new(spawn_service), apps: AppSupervisor::new(spawn_app, create_window), cursor_x: 0, cursor_y: 0 }
    }
    pub fn attach_framebuffer(&mut self, fb: FramebufferInfo) -> Result<(), super::EndUserRuntimeError> { self.core.attach_framebuffer(fb) }
    pub fn register_core_services(&mut self) -> Result<(), awe_initd::RuntimeError> {
        let n = [None; awe_initd::MAX_DEPENDENCIES];
        let d1 = [Some(ServiceId(1)), None, None, None, None, None, None, None];
        let d4 = [Some(ServiceId(4)), None, None, None, None, None, None, None];
        let specs = [
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(1), restart: RestartPolicy::Always, capability_mask: CapabilitySet::DEVICE.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: n, dependency_count: 0, entry: 1 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(2), restart: RestartPolicy::Always, capability_mask: CapabilitySet::STORAGE.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: d1, dependency_count: 1, entry: 2 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(3), restart: RestartPolicy::Always, capability_mask: CapabilitySet::NETWORK.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: d1, dependency_count: 1, entry: 3 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(4), restart: RestartPolicy::Always, capability_mask: CapabilitySet::IPC.0, memory_limit_pages: 32, cpu_budget_ticks: 5_000 }, dependencies: d1, dependency_count: 1, entry: 4 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(5), restart: RestartPolicy::Always, capability_mask: CapabilitySet::PROCESS.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: d4, dependency_count: 1, entry: 5 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(6), restart: RestartPolicy::Always, capability_mask: CapabilitySet::UI.union(CapabilitySet::IPC).0, memory_limit_pages: 128, cpu_budget_ticks: 20_000 }, dependencies: d4, dependency_count: 1, entry: 6 },
        ];
        for spec in specs { self.services.register(spec)?; }
        Ok(())
    }
    pub fn start_core_services(&mut self) -> Result<(), awe_initd::RuntimeError> { for id in 1..=6 { self.services.start(ServiceId(id))?; } Ok(()) }
    pub fn admit_core_apps(&mut self) -> Result<(), awe_appd::AppRuntimeError> {
        let apps = [
            (1u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)),
            (2u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)),
            (3u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)),
            (4u64, CapabilitySet::UI.union(CapabilitySet::IPC)),
        ];
        for (id, caps) in apps {
            let manifest = AppManifest { id: AppId(id), abi_major: awe_appd::AWE_APP_ABI_MAJOR, abi_minor: awe_appd::AWE_APP_ABI_MINOR, memory_limit_pages: 32, capability_mask: caps.0, dependency_count: 0, resource_count: 0 };
            self.apps.admit(manifest).map_err(|_| awe_appd::AppRuntimeError::InvalidManifest)?;
        }
        Ok(())
    }
    pub fn start_core_apps(&mut self) -> Result<(), awe_appd::AppRuntimeError> { for id in 1..=4 { self.apps.start(AppId(id), ALL.0)?; } Ok(()) }
    pub fn route_ps2(&mut self, event: Ps2Event) -> Result<RuntimeEvent, super::EndUserRuntimeError> {
        let translated = match event {
            Ps2Event::Key { code, pressed } => InputEvent::Key { code: key_code_value(code), pressed },
            Ps2Event::Pointer { dx, dy, buttons } => {
                self.cursor_x = self.cursor_x.saturating_add(dx as i32);
                self.cursor_y = self.cursor_y.saturating_add(dy as i32);
                InputEvent::Pointer { x: self.cursor_x, y: self.cursor_y, buttons }
            }
        };
        self.core.push_input(translated)?;
        self.windows.handle_input(translated);
        Ok(RuntimeEvent::Input(translated))
    }
    pub fn create_native_window(&mut self, rect: RuntimeRect) -> Result<u16, WindowError> { self.windows.create(rect) }
    pub fn service_state(&self, id: u16) -> Option<ServiceState> { self.services.state(ServiceId(id)) }
    pub fn app_state(&self, id: u64) -> Option<AppState> { self.apps.state(AppId(id)) }
}

fn key_code_value(code: KeyCode) -> u16 {
    match code {
        KeyCode::Escape => 0x01, KeyCode::Enter => 0x1C, KeyCode::Backspace => 0x0E,
        KeyCode::Tab => 0x0D, KeyCode::Space => 0x39, KeyCode::Left => 0x4B,
        KeyCode::Right => 0x4D, KeyCode::Up => 0x48, KeyCode::Down => 0x50,
        KeyCode::Character(v) | KeyCode::Unknown(v) => v as u16,
    }
}

impl Default for SystemRuntime { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn core_services_start_in_dependency_order() { let mut r = SystemRuntime::new(); r.register_core_services().unwrap(); r.start_core_services().unwrap(); assert_eq!(r.service_state(6), Some(ServiceState::Running)); }
    #[test] fn core_apps_admit_and_start() { let mut r = SystemRuntime::new(); r.admit_core_apps().unwrap(); r.start_core_apps().unwrap(); assert_eq!(r.app_state(1), Some(AppState::Running)); }
    #[test] fn ps2_input_reaches_window_manager() { let mut r = SystemRuntime::new(); r.create_native_window(RuntimeRect { x: 0, y: 0, width: 100, height: 100 }).unwrap(); r.route_ps2(Ps2Event::Pointer { dx: 20, dy: 20, buttons: 1 }).unwrap(); assert!(r.pointer_target().is_some()); }
    #[test] fn capability_set_is_used_for_app_admission() { let mut r = SystemRuntime::new(); r.admit_core_apps().unwrap(); assert_eq!(r.apps.start(AppId(1), CapabilitySet::UI.0), Err(awe_appd::AppRuntimeError::CapabilityDenied)); }
    #[test] fn runtime_context_requires_capability() { assert!(RuntimeContext::new(CapabilitySet::UI).require(CapabilitySet::UI).is_ok()); }
}
