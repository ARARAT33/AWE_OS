#![no_std]

use core::sync::atomic::{AtomicU16, Ordering};
use awe_appd::{AppId, AppManifest, AppState, AppSupervisor};
use awe_initd::{RestartPolicy, ServiceId, ServiceRuntimeSpec, ServiceSpec, ServiceState, Supervisor};
use awe_netd::NetworkDaemon;
use awe_storaged::{StorageDaemon, MAX_PACKAGE_FILES as STORAGE_MAX_PACKAGE_FILES};
use crate::drivers::{KeyCode, Ps2Event};
use crate::process::{ProcessDescriptor, ProcessId, ProcessManager, ProcessState, ResourceBudget};
use crate::process::context::{CpuContext, ProcessContext};
use crate::storage::{Namespace, NamespaceManager, MAX_FILES};
use super::{CapabilitySet, EndUserRuntime, FramebufferInfo, InputEvent, RuntimeEvent};
pub use super::{RuntimeRect, WindowManager, WindowError};

const MAX_RUNTIME_PROCESSES: usize = 32;
static NEXT_WINDOW_ID: AtomicU16 = AtomicU16::new(1);
static mut PROCESS_STACKS: [[u8; 16384]; MAX_RUNTIME_PROCESSES] = [[0; 16384]; MAX_RUNTIME_PROCESSES];

extern "C" fn runtime_service_entry() -> ! { loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); } } }
extern "C" fn runtime_app_entry() -> ! { loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); } } }

fn entry_address(entry: extern "C" fn() -> !) -> usize { entry as *const () as usize }
fn spawn_service(entry: usize, service: ServiceId, memory_pages: u32, cpu_budget: u32) -> Result<u64, ()> { if entry == 0 || service.0 == 0 || memory_pages == 0 || cpu_budget == 0 { return Err(()); } Ok(service.0 as u64) }
fn spawn_app(app: AppId, memory_pages: u32, _capabilities: u64) -> Result<u64, ()> { if app.0 == 0 || memory_pages == 0 { return Err(()); } Ok(0x1_0000 + app.0) }
fn create_window(_app: AppId) -> Result<u16, ()> { let id = NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed); if id == 0 { Err(()) } else { Ok(id) } }

const ALL: CapabilitySet = CapabilitySet(CapabilitySet::PROCESS.0 | CapabilitySet::MEMORY.0 | CapabilitySet::IPC.0 | CapabilitySet::DEVICE.0 | CapabilitySet::STORAGE.0 | CapabilitySet::NETWORK.0 | CapabilitySet::UI.0);

pub struct SystemRuntime {
    pub core: EndUserRuntime,
    pub windows: WindowManager,
    pub services: Supervisor,
    pub apps: AppSupervisor,
    pub processes: ProcessManager<MAX_RUNTIME_PROCESSES>,
    pub namespaces: NamespaceManager<MAX_FILES>,
    pub storage: StorageDaemon,
    pub storage_state: [u8; awe_storaged::persistence::MAX_STATE_SIZE],
    pub storage_state_len: usize,
    pub network: NetworkDaemon,
    pub cursor_x: i32,
    pub cursor_y: i32,
}

impl SystemRuntime {
    pub const fn new() -> Self {
        Self { core: EndUserRuntime::new(), windows: WindowManager::new(), services: Supervisor::new(spawn_service), apps: AppSupervisor::new(spawn_app, create_window), processes: ProcessManager::new(), namespaces: NamespaceManager::new(), storage: StorageDaemon::new(), storage_state: [0; awe_storaged::persistence::MAX_STATE_SIZE], storage_state_len: 0, network: NetworkDaemon::new(), cursor_x: 0, cursor_y: 0 }
    }

    fn register_process(&mut self, pid: u64, entry: u64, memory_pages: u32, ipc_messages: u64) -> Result<(), ()> {
        if pid == 0 || entry == 0 || self.processes.len() >= MAX_RUNTIME_PROCESSES { return Err(()); }
        let slot = self.processes.len();
        let stack_ptr = unsafe { core::ptr::addr_of_mut!(PROCESS_STACKS[slot]) as *mut u8 };
        let stack_top = stack_ptr as u64 + 16384;
        let descriptor = ProcessDescriptor { id: ProcessId(pid), state: ProcessState::Created, budget: ResourceBudget { cpu_ticks: 10_000, memory_bytes: memory_pages as u64 * 4096, ipc_messages } };
        let context = ProcessContext::new(ProcessId(pid), CpuContext::kernel_entry(entry, stack_top, 0));
        self.processes.register(descriptor, context).map_err(|_| ())?;
        self.processes.make_runnable(ProcessId(pid)).map_err(|_| ())
    }

    pub fn attach_framebuffer(&mut self, fb: FramebufferInfo) -> Result<(), super::EndUserRuntimeError> { self.core.attach_framebuffer(fb) }

    pub fn mount_core_namespaces(&mut self, first_block: u64) -> Result<(), crate::storage::NamespaceError> {
        let spacing = 128u64;
        self.namespaces.mount(Namespace::Config, first_block)?;
        self.namespaces.mount(Namespace::Home, first_block + spacing)?;
        self.namespaces.mount(Namespace::Apps, first_block + spacing * 2)?;
        self.namespaces.mount(Namespace::System, first_block + spacing * 3)?;
        self.namespaces.mount(Namespace::Log, first_block + spacing * 4)?;
        let volume = self.storage.register_volume(awe_storaged::VolumeType::AweFsVolume, 8192, first_block, false).map_err(|_| crate::storage::NamespaceError::Invalid)?;
        self.storage.mount_volume(volume, 0xAWE0_0001).map_err(|_| crate::storage::NamespaceError::Invalid)?;
        let _ = self.storage.create_snapshot(volume, 0).map_err(|_| crate::storage::NamespaceError::Invalid)?;
        self.persist_storage_state().map_err(|_| crate::storage::NamespaceError::Invalid)?;
        Ok(())
    }

    pub fn persist_storage_state(&mut self) -> Result<usize, awe_storaged::persistence::PersistError> {
        let len = awe_storaged::persistence::export_state(&self.storage, &mut self.storage_state)?;
        self.storage_state_len = len;
        Ok(len)
    }

    pub fn restore_storage_state(&mut self) -> Result<(), awe_storaged::persistence::PersistError> {
        if self.storage_state_len == 0 { return Ok(()); }
        awe_storaged::persistence::import_state(&mut self.storage, &self.storage_state[..self.storage_state_len])
    }

    pub fn register_network_interface(&mut self, mac: [u8; 6]) -> Result<usize, &'static str> { self.network.add_interface(awe_netd::MacAddress(mac)) }

    pub fn register_core_services(&mut self) -> Result<(), awe_initd::RuntimeError> {
        let n = [None; awe_initd::runtime::MAX_DEPENDENCIES];
        let d1 = [Some(ServiceId(1)), None, None, None, None, None, None, None];
        let d4 = [Some(ServiceId(4)), None, None, None, None, None, None, None];
        let entry = entry_address(runtime_service_entry);
        let specs = [
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(1), restart: RestartPolicy::Always, capability_mask: CapabilitySet::DEVICE.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: n, dependency_count: 0, entry },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(2), restart: RestartPolicy::Always, capability_mask: CapabilitySet::STORAGE.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: d1, dependency_count: 1, entry },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(3), restart: RestartPolicy::Always, capability_mask: CapabilitySet::NETWORK.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: d1, dependency_count: 1, entry },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(4), restart: RestartPolicy::Always, capability_mask: CapabilitySet::IPC.0, memory_limit_pages: 32, cpu_budget_ticks: 5_000 }, dependencies: d1, dependency_count: 1, entry },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(5), restart: RestartPolicy::Always, capability_mask: CapabilitySet::PROCESS.union(CapabilitySet::IPC).0, memory_limit_pages: 64, cpu_budget_ticks: 10_000 }, dependencies: d4, dependency_count: 1, entry },
            ServiceRuntimeSpec { spec: ServiceSpec { id: ServiceId(6), restart: RestartPolicy::Always, capability_mask: CapabilitySet::UI.union(CapabilitySet::IPC).0, memory_limit_pages: 128, cpu_budget_ticks: 20_000 }, dependencies: d4, dependency_count: 1, entry },
        ];
        for spec in specs { self.services.register(spec)?; }
        Ok(())
    }

    pub fn start_core_services(&mut self) -> Result<(), awe_initd::RuntimeError> {
        let entry = entry_address(runtime_service_entry) as u64;
        for id in 1..=6 { let pid = self.services.start(ServiceId(id))?; if self.register_process(pid, entry, 64, 128).is_err() { return Err(awe_initd::RuntimeError::SpawnFailed); } }
        Ok(())
    }

    pub fn admit_core_apps(&mut self) -> Result<(), awe_appd::AppRuntimeError> {
        let apps = [(1u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)), (2u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)), (3u64, CapabilitySet::UI.union(CapabilitySet::IPC).union(CapabilitySet::STORAGE)), (4u64, CapabilitySet::UI.union(CapabilitySet::IPC))];
        for (id, caps) in apps { let manifest = AppManifest { id: AppId(id), abi_major: awe_appd::AWE_APP_ABI_MAJOR, abi_minor: awe_appd::AWE_APP_ABI_MINOR, memory_limit_pages: 32, capability_mask: caps.0, dependency_count: 0, resource_count: 0 }; self.apps.admit(manifest).map_err(|_| awe_appd::AppRuntimeError::InvalidManifest)?; }
        Ok(())
    }

    pub fn start_core_apps(&mut self) -> Result<(), awe_appd::AppRuntimeError> {
        let entry = entry_address(runtime_app_entry) as u64;
        for id in 1..=4 { let pid = self.apps.start(AppId(id), ALL.0)?; if self.register_process(pid, entry, 32, 64).is_err() { return Err(awe_appd::AppRuntimeError::SpawnFailed); } }
        Ok(())
    }

    pub fn route_ps2(&mut self, event: Ps2Event) -> Result<RuntimeEvent, super::EndUserRuntimeError> {
        let translated = match event { Ps2Event::Key { code, pressed } => InputEvent::Key { code: key_code_value(code), pressed }, Ps2Event::Pointer { dx, dy, buttons } => { self.cursor_x = self.cursor_x.saturating_add(dx as i32); self.cursor_y = self.cursor_y.saturating_add(dy as i32); InputEvent::Pointer { x: self.cursor_x, y: self.cursor_y, buttons } } };
        self.core.push_input(translated)?;
        self.windows.handle_input(translated);
        Ok(RuntimeEvent::Input(translated))
    }

    pub fn create_native_window(&mut self, rect: RuntimeRect) -> Result<u16, WindowError> { self.windows.create(rect) }
    pub fn pointer_target(&mut self) -> Option<u16> { self.windows.hit_test(self.cursor_x, self.cursor_y) }
    pub fn service_state(&self, id: u16) -> Option<ServiceState> { self.services.state(ServiceId(id)) }
    pub fn app_state(&self, id: u64) -> Option<AppState> { self.apps.state(AppId(id)) }
    pub fn process_count(&self) -> usize { self.processes.len() }
    pub fn scheduler_ticks(&self) -> u64 { self.processes.scheduler_ticks() }
}

fn key_code_value(code: KeyCode) -> u16 { match code { KeyCode::Escape => 0x01, KeyCode::Enter => 0x1C, KeyCode::Backspace => 0x0E, KeyCode::Tab => 0x0D, KeyCode::Space => 0x39, KeyCode::Left => 0x4B, KeyCode::Right => 0x4D, KeyCode::Up => 0x48, KeyCode::Down => 0x50, KeyCode::Character(v) | KeyCode::Unknown(v) => v as u16 } }

impl Default for SystemRuntime { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn core_services_start_and_register_processes() { let mut r=SystemRuntime::new(); r.register_core_services().unwrap(); r.start_core_services().unwrap(); assert_eq!(r.service_state(6),Some(ServiceState::Running)); assert_eq!(r.process_count(),6); }
    #[test] fn core_apps_admit_and_register_processes() { let mut r=SystemRuntime::new(); r.admit_core_apps().unwrap(); r.start_core_apps().unwrap(); assert_eq!(r.app_state(1),Some(AppState::Running)); assert_eq!(r.process_count(),4); }
    #[test] fn ps2_input_reaches_window_manager() { let mut r=SystemRuntime::new(); r.create_native_window(RuntimeRect{x:0,y:0,width:100,height:100}).unwrap(); r.route_ps2(Ps2Event::Pointer{dx:20,dy:20,buttons:1}).unwrap(); assert!(r.pointer_target().is_some()); }
    #[test] fn capability_set_is_used_for_app_admission() { let mut r=SystemRuntime::new(); r.admit_core_apps().unwrap(); assert_eq!(r.apps.start(AppId(1),CapabilitySet::UI.0),Err(awe_appd::AppRuntimeError::CapabilityDenied)); }
    #[test] fn namespace_mounts_are_bounded_and_storage_persisted() { let mut r=SystemRuntime::new(); r.mount_core_namespaces(32).unwrap(); assert!(r.namespaces.is_mounted(Namespace::Config)); assert!(r.namespaces.is_mounted(Namespace::Log)); assert!(r.storage_state_len > 0); let state_len = r.storage_state_len; assert!(state_len <= awe_storaged::persistence::MAX_STATE_SIZE); assert!(STORAGE_MAX_PACKAGE_FILES > 0); }
}
