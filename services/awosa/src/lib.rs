#![no_std]

//! Stable AWOSA native runtime contract.
//! Concrete execution remains in userspace; this crate defines bounded,
//! capability-aware ABI rules that can be exercised independently.

pub const AWOSA_ABI_MAJOR: u16 = 1;
pub const AWOSA_ABI_MINOR: u16 = 2;
pub const MAX_PATH: usize = 256;
pub const MAX_MESSAGE: usize = 4096;
pub const MAX_IO: usize = 64 * 1024;
pub const MAX_HANDLES: usize = 64;

pub const CAP_FS_READ: u64 = 1 << 0;
pub const CAP_FS_WRITE: u64 = 1 << 1;
pub const CAP_NET: u64 = 1 << 2;
pub const CAP_IPC: u64 = 1 << 3;
pub const CAP_DEVICE: u64 = 1 << 4;
pub const CAP_UI: u64 = 1 << 5;
pub const CAP_KNOWN_MASK: u64 =
    CAP_FS_READ | CAP_FS_WRITE | CAP_NET | CAP_IPC | CAP_DEVICE | CAP_UI;

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
    ResourceExhausted,
    NotFound,
    AlreadyExists,
    UnknownCapability,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_future_major_abi() {
        assert!(negotiate(AbiVersion { major: 2, minor: 0 }).is_err());
    }
    #[test]
    fn accepts_compatible_minor() {
        assert_eq!(
            negotiate(AbiVersion { major: 1, minor: 2 }).unwrap().major,
            1
        );
    }
    #[test]
    fn rejects_unknown_capability_bits() {
        assert_eq!(
            validate_capabilities(1 << 63),
            Err(RuntimeError::UnknownCapability)
        );
    }
    #[test]
    fn rejects_write_without_capability() {
        assert_eq!(
            validate_io(IoKind::Write, 128, CAP_FS_READ),
            Err(RuntimeError::CapabilityDenied)
        );
    }
    #[test]
    fn accepts_bounded_authorized_read() {
        assert!(validate_io(IoKind::Read, 4096, CAP_FS_READ).is_ok());
    }
    #[test]
    fn rejects_zero_sized_io() {
        assert_eq!(
            validate_io(IoKind::Message, 0, CAP_IPC),
            Err(RuntimeError::ResourceExhausted)
        );
    }
    #[test]
    fn handle_table_is_bounded_and_reusable() {
        let mut table = HandleTable::new();
        let h = table.allocate().unwrap();
        assert!(table.contains(h));
        assert_eq!(table.release(h), Ok(()));
        assert!(!table.contains(h));
        assert_eq!(table.release(h), Err(RuntimeError::NotFound));
    }
}
