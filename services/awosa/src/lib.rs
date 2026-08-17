#![no_std]

//! Stable AWOSA native runtime contract. Implementations may evolve behind
//! this versioned ABI without adding a kernel dependency.

pub const AWOSA_ABI_MAJOR: u16 = 1;
pub const AWOSA_ABI_MINOR: u16 = 0;
pub const MAX_PATH: usize = 256;
pub const MAX_MESSAGE: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbiVersion { pub major: u16, pub minor: u16 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError { IncompatibleAbi, InvalidArgument, CapabilityDenied, ResourceExhausted, NotFound }

pub const fn negotiate(requested: AbiVersion) -> Result<AbiVersion, RuntimeError> {
    if requested.major != AWOSA_ABI_MAJOR || requested.minor > AWOSA_ABI_MINOR {
        Err(RuntimeError::IncompatibleAbi)
    } else {
        Ok(AbiVersion { major: AWOSA_ABI_MAJOR, minor: requested.minor })
    }
}

pub const fn validate_path(path_len: usize) -> Result<(), RuntimeError> {
    if path_len == 0 || path_len > MAX_PATH { Err(RuntimeError::InvalidArgument) } else { Ok(()) }
}

pub const fn validate_message(size: usize) -> Result<(), RuntimeError> {
    if size > MAX_MESSAGE { Err(RuntimeError::ResourceExhausted) } else { Ok(()) }
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
        assert_eq!(negotiate(AbiVersion { major: 1, minor: 0 }).unwrap().major, 1);
    }
}
