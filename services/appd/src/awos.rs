//! Bounded `.awos` native application package lifecycle.
//! Admission validates structure first; signature verification is delegated to
//! the platform trust service so package parsing never becomes trusted code.

pub const AWOS_MAGIC: [u8; 4] = *b"AWOS";
pub const AWOS_VERSION: u16 = 1;
pub const AWOS_HEADER_LEN: usize = 32;
pub const AWOS_MAX_MANIFEST: usize = 64 * 1024;
pub const AWOS_MAX_CODE: usize = 256 * 1024 * 1024;
pub const AWOS_MAX_DATA: usize = 256 * 1024 * 1024;
pub const AWOS_MIN_SIGNATURE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AwosHeader {
    pub version: u16,
    pub abi_major: u16,
    pub abi_minor: u16,
    pub manifest_len: u32,
    pub code_len: u32,
    pub data_len: u32,
    pub signature_len: u16,
    pub entry_offset: u32,
    pub flags: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AwosError {
    TooShort,
    BadMagic,
    UnsupportedVersion,
    OversizedManifest,
    OversizedCode,
    OversizedData,
    MissingSignature,
    InvalidLength,
    InvalidEntry,
}

pub fn validate_awos(bytes: &[u8]) -> Result<AwosHeader, AwosError> {
    if bytes.len() < AWOS_HEADER_LEN {
        return Err(AwosError::TooShort);
    }
    if bytes[..4] != AWOS_MAGIC {
        return Err(AwosError::BadMagic);
    }
    let u16_at = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
    let u32_at =
        |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let header = AwosHeader {
        version: u16_at(4),
        abi_major: u16_at(6),
        abi_minor: u16_at(8),
        manifest_len: u32_at(10),
        code_len: u32_at(14),
        data_len: u32_at(18),
        signature_len: u16_at(22),
        entry_offset: u32_at(24),
        flags: u32_at(28),
    };
    if header.version != AWOS_VERSION {
        return Err(AwosError::UnsupportedVersion);
    }
    if header.manifest_len as usize > AWOS_MAX_MANIFEST {
        return Err(AwosError::OversizedManifest);
    }
    if header.code_len as usize > AWOS_MAX_CODE {
        return Err(AwosError::OversizedCode);
    }
    if header.data_len as usize > AWOS_MAX_DATA {
        return Err(AwosError::OversizedData);
    }
    if (header.signature_len as usize) < AWOS_MIN_SIGNATURE {
        return Err(AwosError::MissingSignature);
    }
    if header.entry_offset >= header.code_len || header.code_len == 0 {
        return Err(AwosError::InvalidEntry);
    }
    let expected = AWOS_HEADER_LEN
        .checked_add(header.manifest_len as usize)
        .and_then(|v| v.checked_add(header.code_len as usize))
        .and_then(|v| v.checked_add(header.data_len as usize))
        .and_then(|v| v.checked_add(header.signature_len as usize))
        .ok_or(AwosError::InvalidLength)?;
    if expected != bytes.len() {
        return Err(AwosError::InvalidLength);
    }
    Ok(header)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppPackageState {
    Installed,
    Running,
    Staged,
    Failed,
    Quarantined,
    Removed,
}

pub const fn package_transition(from: AppPackageState, to: AppPackageState) -> bool {
    matches!(
        (from, to),
        (AppPackageState::Installed, AppPackageState::Running)
            | (AppPackageState::Installed, AppPackageState::Staged)
            | (AppPackageState::Running, AppPackageState::Staged)
            | (AppPackageState::Running, AppPackageState::Failed)
            | (AppPackageState::Staged, AppPackageState::Running)
            | (AppPackageState::Staged, AppPackageState::Failed)
            | (AppPackageState::Failed, AppPackageState::Staged)
            | (AppPackageState::Failed, AppPackageState::Quarantined)
            | (AppPackageState::Installed, AppPackageState::Removed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> Vec<u8> {
        let manifest = 4usize;
        let code = 8usize;
        let data = 2usize;
        let sig = 64usize;
        let mut b = Vec::with_capacity(AWOS_HEADER_LEN + manifest + code + data + sig);
        b.extend_from_slice(&AWOS_MAGIC);
        b.extend_from_slice(&AWOS_VERSION.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&0u16.to_le_bytes());
        b.extend_from_slice(&(manifest as u32).to_le_bytes());
        b.extend_from_slice(&(code as u32).to_le_bytes());
        b.extend_from_slice(&(data as u32).to_le_bytes());
        b.extend_from_slice(&(sig as u16).to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend(core::iter::repeat_n(0u8, manifest + code + data + sig));
        b
    }

    #[test]
    fn validates_package_before_admission() {
        assert!(validate_awos(&package()).is_ok());
    }

    #[test]
    fn invalid_entry_is_rejected() {
        let mut b = package();
        b[24..28].copy_from_slice(&8u32.to_le_bytes());
        assert_eq!(validate_awos(&b), Err(AwosError::InvalidEntry));
    }
}
