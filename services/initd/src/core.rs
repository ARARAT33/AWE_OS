//! Bounded userspace core-service contracts for the Stage G execution plane.
//! These contracts keep loader, device/filesystem/network managers, logging,
//! security policy and crash recovery explicit and allocation-free.

use crate::{ServiceId, ServiceState, ServiceTable};

pub const MAX_PATH: usize = 192;
pub const MAX_LOG_MESSAGE: usize = 128;
pub const MAX_CORE_MANAGERS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreError {
    InvalidPath,
    InvalidImage,
    Full,
    Duplicate,
    CapabilityDenied,
    InvalidState,
    RecoveryRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundedPath {
    bytes: [u8; MAX_PATH],
    len: u16,
}

impl BoundedPath {
    pub fn new(path: &[u8]) -> Result<Self, CoreError> {
        if path.is_empty() || path.len() > MAX_PATH || path[0] != b'/' {
            return Err(CoreError::InvalidPath);
        }
        let mut bytes = [0u8; MAX_PATH];
        bytes[..path.len()].copy_from_slice(path);
        Ok(Self {
            bytes,
            len: path.len() as u16,
        })
    }

    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserImage {
    pub entry: u64,
    pub image_len: u32,
    pub required_caps: u64,
}

pub const fn validate_user_image(image: UserImage, granted_caps: u64) -> Result<(), CoreError> {
    if image.entry == 0 || image.image_len == 0 {
        return Err(CoreError::InvalidImage);
    }
    if image.required_caps & !granted_caps != 0 {
        return Err(CoreError::CapabilityDenied);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreManagerKind {
    Device,
    Filesystem,
    Network,
    Logging,
    Security,
    CrashRecovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreManager {
    pub id: ServiceId,
    pub kind: CoreManagerKind,
    pub capabilities: u64,
}

pub struct CoreManagerRegistry {
    entries: [Option<CoreManager>; MAX_CORE_MANAGERS],
    len: usize,
}

impl CoreManagerRegistry {
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_CORE_MANAGERS],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn register(&mut self, manager: CoreManager) -> Result<(), CoreError> {
        if self
            .entries
            .iter()
            .flatten()
            .any(|entry| entry.id == manager.id || entry.kind == manager.kind)
        {
            return Err(CoreError::Duplicate);
        }
        if self.len == MAX_CORE_MANAGERS {
            return Err(CoreError::Full);
        }
        self.entries[self.len] = Some(manager);
        self.len += 1;
        Ok(())
    }

    pub fn get(&self, kind: CoreManagerKind) -> Option<CoreManager> {
        self.entries
            .iter()
            .flatten()
            .find(|entry| entry.kind == kind)
            .copied()
    }
}

impl Default for CoreManagerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogRecord {
    pub sequence: u64,
    pub level: LogLevel,
    pub service: ServiceId,
    pub message: [u8; MAX_LOG_MESSAGE],
    pub message_len: u16,
}

impl LogRecord {
    pub fn new(
        sequence: u64,
        level: LogLevel,
        service: ServiceId,
        message: &[u8],
    ) -> Result<Self, CoreError> {
        if message.len() > MAX_LOG_MESSAGE {
            return Err(CoreError::InvalidPath);
        }
        let mut record = Self {
            sequence,
            level,
            service,
            message: [0; MAX_LOG_MESSAGE],
            message_len: message.len() as u16,
        };
        record.message[..message.len()].copy_from_slice(message);
        Ok(record)
    }

    pub fn message(&self) -> &[u8] {
        &self.message[..self.message_len as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecurityPolicy {
    pub allowed_caps: u64,
    pub deny_by_default: bool,
}

impl SecurityPolicy {
    pub const fn deny_all() -> Self {
        Self {
            allowed_caps: 0,
            deny_by_default: true,
        }
    }

    pub const fn authorize(&self, requested: u64) -> Result<(), CoreError> {
        if self.deny_by_default && requested & !self.allowed_caps != 0 {
            return Err(CoreError::CapabilityDenied);
        }
        if requested & !self.allowed_caps != 0 {
            return Err(CoreError::CapabilityDenied);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashRecord {
    pub service: ServiceId,
    pub state: ServiceState,
    pub fault_code: u32,
    pub recoverable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryAction {
    Restart,
    Quarantine,
    Halt,
}

pub const fn recovery_action(record: CrashRecord) -> RecoveryAction {
    if !record.recoverable {
        return RecoveryAction::Halt;
    }
    match record.state {
        ServiceState::Failed => RecoveryAction::Restart,
        ServiceState::Quarantined => RecoveryAction::Quarantine,
        _ => RecoveryAction::Halt,
    }
}

pub fn start_core_manager(
    services: &mut ServiceTable,
    manager: CoreManager,
) -> Result<(), CoreError> {
    let Some(state) = services.state(manager.id) else {
        return Err(CoreError::InvalidState);
    };
    if state == ServiceState::Failed {
        services
            .restart(manager.id)
            .map_err(|_| CoreError::RecoveryRequired)?;
        return Ok(());
    }
    if state == ServiceState::Declared {
        services
            .set_state(manager.id, ServiceState::Starting)
            .map_err(|_| CoreError::InvalidState)?;
        services
            .set_state(manager.id, ServiceState::Running)
            .map_err(|_| CoreError::InvalidState)?;
        return Ok(());
    }
    if state == ServiceState::Running {
        return Ok(());
    }
    Err(CoreError::InvalidState)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RestartPolicy, ServiceSpec};

    #[test]
    fn loader_rejects_missing_entry_and_caps() {
        assert_eq!(
            validate_user_image(
                UserImage {
                    entry: 0,
                    image_len: 1,
                    required_caps: 0
                },
                0
            ),
            Err(CoreError::InvalidImage)
        );
        assert_eq!(
            validate_user_image(
                UserImage {
                    entry: 1,
                    image_len: 1,
                    required_caps: 4
                },
                2
            ),
            Err(CoreError::CapabilityDenied)
        );
    }

    #[test]
    fn manager_registry_is_bounded_and_unique() {
        let mut registry = CoreManagerRegistry::new();
        let manager = CoreManager {
            id: ServiceId(1),
            kind: CoreManagerKind::Network,
            capabilities: 1,
        };
        registry.register(manager).unwrap();
        assert_eq!(registry.get(CoreManagerKind::Network), Some(manager));
        assert_eq!(registry.register(manager), Err(CoreError::Duplicate));
    }

    #[test]
    fn security_is_deny_by_default() {
        let policy = SecurityPolicy::deny_all();
        assert_eq!(policy.authorize(1), Err(CoreError::CapabilityDenied));
    }

    #[test]
    fn crash_policy_restarts_only_recoverable_failed_services() {
        let record = CrashRecord {
            service: ServiceId(2),
            state: ServiceState::Failed,
            fault_code: 7,
            recoverable: true,
        };
        assert_eq!(recovery_action(record), RecoveryAction::Restart);
        assert_eq!(
            recovery_action(CrashRecord {
                recoverable: false,
                ..record
            }),
            RecoveryAction::Halt
        );
    }

    #[test]
    fn core_manager_can_start_through_service_table() {
        let mut services = ServiceTable::new();
        services
            .register(ServiceSpec {
                id: ServiceId(3),
                restart: RestartPolicy::OnFailure,
                capability_mask: 1,
                memory_limit_pages: 4,
                cpu_budget_ticks: 10,
            })
            .unwrap();
        start_core_manager(
            &mut services,
            CoreManager {
                id: ServiceId(3),
                kind: CoreManagerKind::Filesystem,
                capabilities: 1,
            },
        )
        .unwrap();
        assert_eq!(services.state(ServiceId(3)), Some(ServiceState::Running));
    }
}
