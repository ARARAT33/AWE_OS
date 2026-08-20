#![no_std]

//! Stable AWOSA Native Application Platform Runtime & ABI Specification.
//!
//! Provides bounded, capability-aware, no_std runtime APIs for native AWOSA applications,
//! covering process, memory, filesystem, networking, UI, device, IPC, capabilities,
//! permissions, async task scheduling, and SDK developer bindings.

pub const AWOSA_ABI_MAJOR: u16 = 1;
pub const AWOSA_ABI_MINOR: u16 = 3;
pub const MAX_PATH: usize = 256;
pub const MAX_MESSAGE: usize = 4096;
pub const MAX_IO: usize = 64 * 1024;
pub const MAX_HANDLES: usize = 64;
pub const MAX_PROCESSES: usize = 32;
pub const MAX_IPC_CHANNELS: usize = 16;
pub const MAX_ASYNC_TASKS: usize = 32;

// --- Capability Bitmasks ---
pub const CAP_NONE: u64 = 0;
pub const CAP_FS_READ: u64 = 1 << 0;
pub const CAP_FS_WRITE: u64 = 1 << 1;
pub const CAP_NET: u64 = 1 << 2;
pub const CAP_IPC: u64 = 1 << 3;
pub const CAP_DEVICE: u64 = 1 << 4;
pub const CAP_UI: u64 = 1 << 5;
pub const CAP_PROCESS_SPAWN: u64 = 1 << 6;
pub const CAP_MEM_ADMIN: u64 = 1 << 7;
pub const CAP_KNOWN_MASK: u64 = CAP_FS_READ
    | CAP_FS_WRITE
    | CAP_NET
    | CAP_IPC
    | CAP_DEVICE
    | CAP_UI
    | CAP_PROCESS_SPAWN
    | CAP_MEM_ADMIN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbiVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IoKind {
    Read,
    Write,
    Message,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    IncompatibleAbi,
    InvalidArgument,
    CapabilityDenied,
    PermissionDenied,
    ResourceExhausted,
    NotFound,
    AlreadyExists,
    UnknownCapability,
    IoError,
    ProcessFailed,
    NotReady,
}

// ============================================================================
// 1. ABI & Capability Negotiation
// ============================================================================

pub const fn negotiate(requested: AbiVersion) -> Result<AbiVersion, RuntimeError> {
    if requested.major != AWOSA_ABI_MAJOR || requested.minor > AWOSA_ABI_MINOR {
        Err(RuntimeError::IncompatibleAbi)
    } else {
        Ok(AbiVersion {
            major: AWOSA_ABI_MAJOR,
            minor: requested.minor,
        })
    }
}

pub const fn validate_capabilities(mask: u64) -> Result<(), RuntimeError> {
    if mask & !CAP_KNOWN_MASK != 0 {
        Err(RuntimeError::UnknownCapability)
    } else {
        Ok(())
    }
}

pub const fn validate_path(path_len: usize) -> Result<(), RuntimeError> {
    if path_len == 0 || path_len > MAX_PATH {
        Err(RuntimeError::InvalidArgument)
    } else {
        Ok(())
    }
}

pub const fn validate_message(size: usize) -> Result<(), RuntimeError> {
    if size == 0 || size > MAX_MESSAGE {
        Err(RuntimeError::ResourceExhausted)
    } else {
        Ok(())
    }
}

pub const fn required_capability(kind: IoKind) -> u64 {
    match kind {
        IoKind::Read => CAP_FS_READ,
        IoKind::Write => CAP_FS_WRITE,
        IoKind::Message => CAP_IPC,
    }
}

pub const fn validate_io(kind: IoKind, size: usize, capabilities: u64) -> Result<(), RuntimeError> {
    if validate_capabilities(capabilities).is_err() {
        return Err(RuntimeError::UnknownCapability);
    }
    let limit = match kind {
        IoKind::Read | IoKind::Write => MAX_IO,
        IoKind::Message => MAX_MESSAGE,
    };
    if size == 0 || size > limit {
        return Err(RuntimeError::ResourceExhausted);
    }
    if capabilities & required_capability(kind) == 0 {
        return Err(RuntimeError::CapabilityDenied);
    }
    Ok(())
}

// ============================================================================
// 2. Handle Table Management
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandleTable {
    used: [bool; MAX_HANDLES],
}

impl HandleTable {
    pub const fn new() -> Self {
        Self {
            used: [false; MAX_HANDLES],
        }
    }

    pub fn allocate(&mut self) -> Result<u16, RuntimeError> {
        let mut i = 0;
        while i < MAX_HANDLES {
            if !self.used[i] {
                self.used[i] = true;
                return Ok(i as u16);
            }
            i += 1;
        }
        Err(RuntimeError::ResourceExhausted)
    }

    pub fn release(&mut self, handle: u16) -> Result<(), RuntimeError> {
        let index = handle as usize;
        if index >= MAX_HANDLES || !self.used[index] {
            return Err(RuntimeError::NotFound);
        }
        self.used[index] = false;
        Ok(())
    }

    pub const fn contains(&self, handle: u16) -> bool {
        let index = handle as usize;
        index < MAX_HANDLES && self.used[index]
    }
}

impl Default for HandleTable {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 3. Process APIs
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Unused,
    Created,
    Running,
    Blocked,
    Terminated(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessControlBlock {
    pub pid: ProcessId,
    pub state: ProcessState,
    pub capabilities: u64,
    pub memory_pages: u32,
    pub parent_pid: u32,
}

pub struct ProcessManager {
    processes: [Option<ProcessControlBlock>; MAX_PROCESSES],
    next_pid: u32,
}

impl ProcessManager {
    pub const fn new() -> Self {
        Self {
            processes: [None; MAX_PROCESSES],
            next_pid: 1,
        }
    }

    pub fn spawn(
        &mut self,
        capabilities: u64,
        memory_pages: u32,
        parent_pid: u32,
        caller_caps: u64,
    ) -> Result<ProcessId, RuntimeError> {
        if caller_caps & CAP_PROCESS_SPAWN == 0 {
            return Err(RuntimeError::CapabilityDenied);
        }
        validate_capabilities(capabilities)?;
        let pid = ProcessId(self.next_pid);

        for slot in self.processes.iter_mut() {
            if slot.is_none() {
                *slot = Some(ProcessControlBlock {
                    pid,
                    state: ProcessState::Created,
                    capabilities,
                    memory_pages,
                    parent_pid,
                });
                self.next_pid += 1;
                return Ok(pid);
            }
        }
        Err(RuntimeError::ResourceExhausted)
    }

    pub fn set_state(&mut self, pid: ProcessId, state: ProcessState) -> Result<(), RuntimeError> {
        for slot in self.processes.iter_mut().flatten() {
            if slot.pid == pid {
                slot.state = state;
                return Ok(());
            }
        }
        Err(RuntimeError::NotFound)
    }

    pub fn get_info(&self, pid: ProcessId) -> Result<ProcessControlBlock, RuntimeError> {
        for slot in self.processes.iter().flatten() {
            if slot.pid == pid {
                return Ok(*slot);
            }
        }
        Err(RuntimeError::NotFound)
    }

    pub fn terminate(&mut self, pid: ProcessId, exit_code: i32) -> Result<(), RuntimeError> {
        self.set_state(pid, ProcessState::Terminated(exit_code))
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 4. Memory APIs
// ============================================================================

pub const PAGE_SIZE: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryRegion {
    pub virt_addr: u64,
    pub page_count: u32,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

pub struct MemoryManager {
    regions: [Option<MemoryRegion>; 16],
    next_vaddr: u64,
}

impl MemoryManager {
    pub const fn new() -> Self {
        Self {
            regions: [None; 16],
            next_vaddr: 0x0040_0000,
        }
    }

    pub fn allocate(
        &mut self,
        page_count: u32,
        writable: bool,
        executable: bool,
        capabilities: u64,
    ) -> Result<MemoryRegion, RuntimeError> {
        if page_count == 0 {
            return Err(RuntimeError::InvalidArgument);
        }
        if executable && (capabilities & CAP_MEM_ADMIN == 0) {
            return Err(RuntimeError::CapabilityDenied);
        }
        let virt_addr = self.next_vaddr;

        for slot in self.regions.iter_mut() {
            if slot.is_none() {
                let region = MemoryRegion {
                    virt_addr,
                    page_count,
                    readable: true,
                    writable,
                    executable,
                };
                *slot = Some(region);
                self.next_vaddr += (page_count as u64) * (PAGE_SIZE as u64);
                return Ok(region);
            }
        }
        Err(RuntimeError::ResourceExhausted)
    }

    pub fn free(&mut self, virt_addr: u64) -> Result<(), RuntimeError> {
        for slot in self.regions.iter_mut() {
            if let Some(reg) = slot {
                if reg.virt_addr == virt_addr {
                    *slot = None;
                    return Ok(());
                }
            }
        }
        Err(RuntimeError::NotFound)
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 5. Filesystem APIs
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileMode {
    Read,
    Write,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileDescriptor {
    pub handle: u16,
    pub mode: FileMode,
    pub offset: u64,
}

pub struct AwosaFilesystemApi {
    pub handles: HandleTable,
    pub descriptors: [Option<FileDescriptor>; MAX_HANDLES],
}

impl AwosaFilesystemApi {
    pub const fn new() -> Self {
        Self {
            handles: HandleTable::new(),
            descriptors: [None; MAX_HANDLES],
        }
    }

    pub fn open(
        &mut self,
        path: &[u8],
        mode: FileMode,
        capabilities: u64,
    ) -> Result<u16, RuntimeError> {
        validate_path(path.len())?;
        let required = match mode {
            FileMode::Read => CAP_FS_READ,
            FileMode::Write => CAP_FS_WRITE,
            FileMode::ReadWrite => CAP_FS_READ | CAP_FS_WRITE,
        };
        if capabilities & required != required {
            return Err(RuntimeError::CapabilityDenied);
        }

        let handle = self.handles.allocate()?;
        self.descriptors[handle as usize] = Some(FileDescriptor {
            handle,
            mode,
            offset: 0,
        });
        Ok(handle)
    }

    pub fn close(&mut self, handle: u16) -> Result<(), RuntimeError> {
        self.handles.release(handle)?;
        self.descriptors[handle as usize] = None;
        Ok(())
    }
}

impl Default for AwosaFilesystemApi {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 6. Networking & Socket APIs
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketProtocol {
    Tcp,
    Udp,
}

pub struct AwosaNetApi {
    socket_handles: HandleTable,
}

impl AwosaNetApi {
    pub const fn new() -> Self {
        Self {
            socket_handles: HandleTable::new(),
        }
    }

    pub fn socket_create(
        &mut self,
        _proto: SocketProtocol,
        capabilities: u64,
    ) -> Result<u16, RuntimeError> {
        if capabilities & CAP_NET == 0 {
            return Err(RuntimeError::CapabilityDenied);
        }
        self.socket_handles.allocate()
    }

    pub fn socket_close(&mut self, handle: u16) -> Result<(), RuntimeError> {
        self.socket_handles.release(handle)
    }
}

impl Default for AwosaNetApi {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 7. UI & Graphics APIs
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title_len: u16,
}

pub struct AwosaUiApi {
    windows: HandleTable,
}

impl AwosaUiApi {
    pub const fn new() -> Self {
        Self {
            windows: HandleTable::new(),
        }
    }

    pub fn create_window(
        &mut self,
        config: WindowConfig,
        capabilities: u64,
    ) -> Result<u16, RuntimeError> {
        if capabilities & CAP_UI == 0 {
            return Err(RuntimeError::CapabilityDenied);
        }
        if config.width == 0 || config.height == 0 {
            return Err(RuntimeError::InvalidArgument);
        }
        self.windows.allocate()
    }

    pub fn close_window(&mut self, handle: u16) -> Result<(), RuntimeError> {
        self.windows.release(handle)
    }
}

impl Default for AwosaUiApi {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 8. IPC & Messaging APIs
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpcChannelDescriptor {
    pub channel_id: u32,
    pub owner_pid: ProcessId,
}

pub struct AwosaIpcApi {
    channels: [Option<IpcChannelDescriptor>; MAX_IPC_CHANNELS],
    next_channel_id: u32,
}

impl AwosaIpcApi {
    pub const fn new() -> Self {
        Self {
            channels: [None; MAX_IPC_CHANNELS],
            next_channel_id: 100,
        }
    }

    pub fn create_channel(
        &mut self,
        owner_pid: ProcessId,
        capabilities: u64,
    ) -> Result<u32, RuntimeError> {
        if capabilities & CAP_IPC == 0 {
            return Err(RuntimeError::CapabilityDenied);
        }
        let cid = self.next_channel_id;

        for slot in self.channels.iter_mut() {
            if slot.is_none() {
                *slot = Some(IpcChannelDescriptor {
                    channel_id: cid,
                    owner_pid,
                });
                self.next_channel_id += 1;
                return Ok(cid);
            }
        }
        Err(RuntimeError::ResourceExhausted)
    }

    pub fn send_message(
        &self,
        channel_id: u32,
        msg: &[u8],
        capabilities: u64,
    ) -> Result<usize, RuntimeError> {
        validate_io(IoKind::Message, msg.len(), capabilities)?;
        for slot in self.channels.iter().flatten() {
            if slot.channel_id == channel_id {
                return Ok(msg.len());
            }
        }
        Err(RuntimeError::NotFound)
    }
}

impl Default for AwosaIpcApi {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 9. Async & Concurrency Engine
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Ready,
    Running,
    Completed(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AsyncTask {
    pub task_id: u32,
    pub status: TaskStatus,
}

pub struct AwosaAsyncScheduler {
    tasks: [Option<AsyncTask>; MAX_ASYNC_TASKS],
    next_task_id: u32,
}

impl AwosaAsyncScheduler {
    pub const fn new() -> Self {
        Self {
            tasks: [None; MAX_ASYNC_TASKS],
            next_task_id: 1,
        }
    }

    pub fn spawn_task(&mut self) -> Result<u32, RuntimeError> {
        let tid = self.next_task_id;
        for slot in self.tasks.iter_mut() {
            if slot.is_none() {
                *slot = Some(AsyncTask {
                    task_id: tid,
                    status: TaskStatus::Ready,
                });
                self.next_task_id += 1;
                return Ok(tid);
            }
        }
        Err(RuntimeError::ResourceExhausted)
    }

    pub fn poll_task(&mut self, task_id: u32) -> Result<TaskStatus, RuntimeError> {
        for slot in self.tasks.iter_mut().flatten() {
            if slot.task_id == task_id {
                if slot.status == TaskStatus::Ready {
                    slot.status = TaskStatus::Running;
                }
                return Ok(slot.status);
            }
        }
        Err(RuntimeError::NotFound)
    }

    pub fn complete_task(&mut self, task_id: u32, result: i32) -> Result<(), RuntimeError> {
        for slot in self.tasks.iter_mut().flatten() {
            if slot.task_id == task_id {
                slot.status = TaskStatus::Completed(result);
                return Ok(());
            }
        }
        Err(RuntimeError::NotFound)
    }
}

impl Default for AwosaAsyncScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 10. SDK Context & Developer Bindings
// ============================================================================

pub struct AwosaSdkContext {
    pub abi_version: AbiVersion,
    pub process_mgr: ProcessManager,
    pub memory_mgr: MemoryManager,
    pub fs_api: AwosaFilesystemApi,
    pub net_api: AwosaNetApi,
    pub ui_api: AwosaUiApi,
    pub ipc_api: AwosaIpcApi,
    pub async_scheduler: AwosaAsyncScheduler,
}

impl AwosaSdkContext {
    pub fn init(requested_abi: AbiVersion) -> Result<Self, RuntimeError> {
        let abi_version = negotiate(requested_abi)?;
        Ok(Self {
            abi_version,
            process_mgr: ProcessManager::new(),
            memory_mgr: MemoryManager::new(),
            fs_api: AwosaFilesystemApi::new(),
            net_api: AwosaNetApi::new(),
            ui_api: AwosaUiApi::new(),
            ipc_api: AwosaIpcApi::new(),
            async_scheduler: AwosaAsyncScheduler::new(),
        })
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_negotiation_and_capabilities() {
        assert!(negotiate(AbiVersion { major: 1, minor: 2 }).is_ok());
        assert_eq!(
            negotiate(AbiVersion { major: 2, minor: 0 }),
            Err(RuntimeError::IncompatibleAbi)
        );
        assert!(validate_capabilities(CAP_FS_READ | CAP_UI).is_ok());
        assert_eq!(
            validate_capabilities(1 << 63),
            Err(RuntimeError::UnknownCapability)
        );
    }

    #[test]
    fn test_handle_table_recycling() {
        let mut table = HandleTable::new();
        let h1 = table.allocate().unwrap();
        let h2 = table.allocate().unwrap();
        assert!(table.contains(h1));
        assert!(table.contains(h2));
        table.release(h1).unwrap();
        assert!(!table.contains(h1));
        let h3 = table.allocate().unwrap();
        assert_eq!(h1, h3);
    }

    #[test]
    fn test_process_lifecycle_and_spawning() {
        let mut pm = ProcessManager::new();
        let pid = pm
            .spawn(
                CAP_FS_READ,
                16,
                0,
                CAP_PROCESS_SPAWN, // Caller has spawn permission
            )
            .unwrap();

        assert_eq!(pm.get_info(pid).unwrap().state, ProcessState::Created);
        pm.set_state(pid, ProcessState::Running).unwrap();
        assert_eq!(pm.get_info(pid).unwrap().state, ProcessState::Running);

        pm.terminate(pid, 0).unwrap();
        assert_eq!(pm.get_info(pid).unwrap().state, ProcessState::Terminated(0));

        // Fail without spawn cap
        assert_eq!(
            pm.spawn(CAP_FS_READ, 16, 0, CAP_NONE),
            Err(RuntimeError::CapabilityDenied)
        );
    }

    #[test]
    fn test_memory_allocation_and_permissions() {
        let mut mm = MemoryManager::new();
        let reg = mm
            .allocate(4, true, false, CAP_NONE)
            .expect("alloc data page");
        assert_eq!(reg.page_count, 4);

        // Executable memory requires CAP_MEM_ADMIN
        assert_eq!(
            mm.allocate(1, false, true, CAP_NONE),
            Err(RuntimeError::CapabilityDenied)
        );
        let exec_reg = mm
            .allocate(1, false, true, CAP_MEM_ADMIN)
            .expect("alloc code page");
        assert!(exec_reg.executable);

        mm.free(reg.virt_addr).unwrap();
        assert_eq!(mm.free(reg.virt_addr), Err(RuntimeError::NotFound));
    }

    #[test]
    fn test_filesystem_and_net_and_ui_and_ipc_apis() {
        let mut sdk = AwosaSdkContext::init(AbiVersion { major: 1, minor: 3 }).unwrap();

        // FS
        let f_handle = sdk
            .fs_api
            .open(b"/app/data.txt", FileMode::Read, CAP_FS_READ)
            .unwrap();
        assert!(sdk.fs_api.close(f_handle).is_ok());

        // Net
        let s_handle = sdk
            .net_api
            .socket_create(SocketProtocol::Tcp, CAP_NET)
            .unwrap();
        assert!(sdk.net_api.socket_close(s_handle).is_ok());

        // UI
        let w_config = WindowConfig {
            width: 800,
            height: 600,
            title_len: 12,
        };
        let w_handle = sdk.ui_api.create_window(w_config, CAP_UI).unwrap();
        assert!(sdk.ui_api.close_window(w_handle).is_ok());

        // IPC
        let cid = sdk.ipc_api.create_channel(ProcessId(1), CAP_IPC).unwrap();
        let sent = sdk.ipc_api.send_message(cid, b"ping", CAP_IPC).unwrap();
        assert_eq!(sent, 4);
    }

    #[test]
    fn test_async_scheduler() {
        let mut sched = AwosaAsyncScheduler::new();
        let tid = sched.spawn_task().unwrap();
        assert_eq!(sched.poll_task(tid).unwrap(), TaskStatus::Running);
        sched.complete_task(tid, 42).unwrap();
        assert_eq!(sched.poll_task(tid).unwrap(), TaskStatus::Completed(42));
    }
}
