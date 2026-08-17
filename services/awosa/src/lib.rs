#![no_std]

//! Stable AWOSA native runtime contract. Implementations may evolve behind
//! this versioned ABI without adding a kernel dependency.

pub const AWOSA_ABI_MAJOR: u16 = 1;
pub const AWOSA_ABI_MINOR: u16 = 1;
pub const MAX_PATH: usize = 256;
pub const MAX_MESSAGE: usize = 4096;
pub const MAX_IO: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbiVersion { pub major: u16, pub minor: u16 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError { IncompatibleAbi, InvalidArgument, CapabilityDenied, ResourceExhausted, NotFound }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability { Process, Memory, Filesystem, Network, Ui, Device, Ipc }

pub const fn negotiate(requested: AbiVersion) -> Result<AbiVersion, RuntimeError> {
    if requested.major != AWOSA_ABI_MAJOR || requested.minor > AWOSA_ABI_MINOR { Err(RuntimeError::IncompatibleAbi) }
    else { Ok(AbiVersion { major: AWOSA_ABI_MAJOR, minor: requested.minor }) }
}

pub const fn validate_path(path_len: usize) -> Result<(), RuntimeError> {
    if path_len == 0 || path_len > MAX_PATH { Err(RuntimeError::InvalidArgument) } else { Ok(()) }
}

pub const fn validate_message(size: usize) -> Result<(), RuntimeError> {
    if size > MAX_MESSAGE { Err(RuntimeError::ResourceExhausted) } else { Ok(()) }
}

pub const fn validate_io(size: usize) -> Result<(), RuntimeError> {
    if size > MAX_IO { Err(RuntimeError::ResourceExhausted) } else { Ok(()) }
}

pub const fn authorize(mask: u64, capability: Capability) -> Result<(), RuntimeError> {
    let bit = match capability { Capability::Process => 1, Capability::Memory => 2, Capability::Filesystem => 4, Capability::Network => 8, Capability::Ui => 16, Capability::Device => 32, Capability::Ipc => 64 };
    if mask & bit != 0 { Ok(()) } else { Err(RuntimeError::CapabilityDenied) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_future_major_abi() { assert!(negotiate(AbiVersion { major: 2, minor: 0 }).is_err()); }
    #[test] fn accepts_compatible_minor() { assert_eq!(negotiate(AbiVersion { major: 1, minor: 1 }).unwrap().major, 1); }
    #[test] fn validates_runtime_boundaries() { assert!(validate_path(1).is_ok()); assert!(validate_message(MAX_MESSAGE + 1).is_err()); assert!(validate_io(MAX_IO + 1).is_err()); }
    #[test] fn capability_api_is_fail_closed() { assert!(authorize(4, Capability::Filesystem).is_ok()); assert!(authorize(4, Capability::Network).is_err()); }
}
