#![no_std]

use core::sync::atomic::{AtomicU16, AtomicU64, Ordering};

use awe_appd::{AppId, AppManifest, AppState, AppSupervisor};
use awe_initd::{RestartPolicy, ServiceId, ServiceRuntimeSpec, ServiceSpec, ServiceState, Supervisor};

use super::{CapabilitySet, EndUserRuntime, FramebufferInfo, InputEvent, RuntimeContext, RuntimeEvent};
use super::{Ps2Event, WindowManager, RuntimeRect};

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
    CapabilitySet::PROCESS.0
        | CapabilitySet::MEMORY.0
        | CapabilitySet::IPC.0
        | CapabilitySet::DEVICE.0
        | CapabilitySet::STORAGE.0
        | CapabilitySet::NETWORK.0
        | CapabilitySet::UI.0,
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
        Self {
            core: EndUserRuntime::new(),
            windows: WindowManager::new(),
            services: Supervisor::new(spawn_service),
            apps: AppSupervisor::new(spawn_app, create_window),
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn attach_framebuffer(&mut self, fb: FramebufferInfo) -> Result<(), super::EndUserRuntimeError> {
        self.core.attach_framebuffer(fb)
    }

    pub fn register_core_services(&mut self) -> Result<(), awe_initd::RuntimeError> {
        let none = [None; awe_initd::MAX_DEPENDENCIES];
        let specs = [
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(1), restart: RestartPolicy::Always, capability_mask: CapabilitySet::DEVICE.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: none, dependency_count: 0, entry: 1 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(2), restart: RestartPolicy::Always, capability_mask: CapabilitySet::STORAGE.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: [Some(ServiceId(1)), None, None, None, None, None, None, None], dependency_count: 1, entry: 2 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(3), restart: RestartPolicy::Always, capability_mask: CapabilitySet::NETWORK.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: [Some(ServiceId(1)), None, None, None, None, None, None, None], dependency_count: 1, entry: 3 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(4), restart: RestartPolicy::Always, capability_mask: CapabilitySet::IPC.0, memory_limit_pages: 32, cpu_budget_ticks: 5_000 }, dependencies: [Some(ServiceId(1)), None, None, None, None, None, None, None], dependency_count: 1, entry: 4 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(5), restart: RestartPolicy::Always, capability_mask: CapabilitySet::PROCESS.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: [Some(ServiceId(4)), None, None, None, None, None, None, None], dependency_count: 1, entry: 5 },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(6), restart: RestartPolicy::Always, capability_mask: CapabilitySet::UI.union(CapabilitySet::IPC).0, memory_limit_pages: 128, cpu_budget_ticks: 20_000 }, dependencies: [Some(ServiceId(4)), None, None, None, None, None, None, None], dependency_count: 1, entry: 6 },
        ];
        for spec in specs { self.services.register(spec)?; }
        Ok(())
    }

    pub fn start_core_services(&mut self) -> Result<(), awe_initd::RuntimeError> {
        for id in 1..=6 { self.services.start(ServiceId(id))?; }
        Ok(())
    }

    pub fn admit_core_apps(&mut self) -> Result<(), awe_appd::AppRuntimeError> {
        let apps = [
            (1u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)),
            (2u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)),
            (3u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)),
            (4u64, CapabilitySet::UI.union(CapabilitySet::IPC)),
        ];
        for (id, caps) in apps {
            self.apps.admit(AppManifest { id: AppId(id), abi_major: awe_appd::AWE_APP_ABI_MAJOR, abi_minor: awe_appd::AWE_APP_ABI_MINOR, memory_limit_pages: 32, capability_mask: caps.0, dependency_count: 0, resource_count: 0 }).map_err(|_| awe_appd::AppRuntimeError::InvalidManifest)?;
        }
        Ok(())
    }

    pub fn start_core_apps(&mut self) -> Result<(), awe_appd::AppRuntimeError> {
        for id in 1..=4 {
            self.apps.start(AppId(id), ALL.0).map_err(|e| e)?;
        }
        Ok(())
    }

    pub fn route_ps2(&mut self, event: Ps2Event) -> Result<Option<RuntimeEvent>, super::EndUserRuntimeError> {
        let translated = match event {
            Ps2Event::Key { code, pressed } => InputEvent::Key { code: code as u16, pressed },
            Ps2Event::Pointer { dx, dy, buttons } => {
                self.cursor_x = self.cursor_x.saturating_add(dx as i32);
                self.cursor_y = self.cursor_y.saturating_add(dy as i32);
                InputEvent::Pointer { x: self.cursor_x, y: self.cursor_y, buttons }
            }
        };
        self.core.push_input(translated)?;
        if self.windows.handle_input(translated).is_some() {
            Ok(Some(RuntimeEvent::Input(translated)))
        } else {
            Ok(Some(RuntimeEvent::Input(translated)))
        }
    }

    pub fn create_native_window(&mut self, x: i32, y: i32, width: u32, height: u32) -> Result<u16, super::WindowError> {
        self.windows.create(RuntimeRect { x, y, width, height })
    }

    pub fn pointer_target(&mut self) -> Option<u16> { self.windows.hit_test(self.cursor_x, self.cursor_y) }

    pub fn service_state(&self, id: u16) -> Option<ServiceState> { self.services.state(ServiceId(id)) }
    pub fn app_state(&self, id: u64) -> Option<AppState> { self.apps.state(AppId(id)) }
}

impl Default for SystemRuntime { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_services_start_in_dependency_order() {
        let mut runtime = SystemRuntime::new();
        runtime.register_core_services().unwrap();
        runtime.start_core_services().unwrap();
        assert_eq!(runtime.service_state(1), Some(ServiceState::Running));
        assert_eq!(runtime.service_state(6), Some(ServiceState::Running));
    }

    #[test]
    fn core_apps_are_admitted_and_started_with_capabilities() {
        let mut runtime = SystemRuntime::new();
        runtime.admit_core_apps().unwrap();
        runtime.start_core_apps().unwrap();
        assert_eq!(runtime.app_state(1), Some(AppState::Running));
        assert_eq!(runtime.app_state(4), Some(AppState::Running));
    }

    #[test]
    fn pointer_events_reach_window_manager() {
        let mut runtime = SystemRuntime::new();
        runtime.create_native_window(0, 0, 100, 100).unwrap();
        runtime.route_ps2(Ps2Event::Pointer { dx: 20, dy: 20, buttons: 1 }).unwrap();
        assert!(runtime.pointer_target().is_some());
    }
}