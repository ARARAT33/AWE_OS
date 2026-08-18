//! Bounded `.asd` native-driver package contract.
//! The package format deliberately separates canonical metadata from the
//! platform signature verifier so cryptographic policy can evolve without
//! changing the driver lifecycle ABI.

pub const ASD_MAGIC: [u8; 4] = *b"ASD1";
pub const ASD_VERSION: u16 = 1;
pub const ASD_HEADER_LEN: usize = 38;
pub const ASD_MAX_MANIFEST: usize = 64 * 1024;
pub const ASD_MAX_PAYLOAD: usize = 128 * 1024 * 1024;
pub const ASD_MIN_SIGNATURE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AsdHeader {
    pub version: u16,
    pub architecture: u16,
    pub manifest_len: u32,
    pub payload_len: u32,
    pub signature_len: u16,
    pub abi_major: u16,
    pub abi_minor: u16,
    pub capabilities: u64,
    pub reserved: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsdError {
    TooShort,
    BadMagic,
    UnsupportedVersion,
    InvalidLength,
    OversizedManifest,
    OversizedPayload,
    MissingSignature,
    ArithmeticOverflow,
}

pub fn validate_asd(bytes: &[u8]) -> Result<AsdHeader, AsdError> {
    if bytes.len() < ASD_HEADER_LEN {
        return Err(AsdError::TooShort);
    }
    if bytes[..4] != ASD_MAGIC {
        return Err(AsdError::BadMagic);
    }
    let u16_at = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
    let u32_at = |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let u64_at = |o: usize| u64::from_le_bytes([
        bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3],
        bytes[o + 4], bytes[o + 5], bytes[o + 6], bytes[o + 7],
    ]);
    let header = AsdHeader {
        version: u16_at(4),
        architecture: u16_at(6),
        manifest_len: u32_at(8),
        payload_len: u32_at(12),
        signature_len: u16_at(16),
        abi_major: u16_at(18),
        abi_minor: u16_at(20),
        capabilities: u64_at(22),
        reserved: u64_at(30),
    };
    if header.version != ASD_VERSION {
        return Err(AsdError::UnsupportedVersion);
    }
    if header.manifest_len as usize > ASD_MAX_MANIFEST {
        return Err(AsdError::OversizedManifest);
    }
    if header.payload_len as usize > ASD_MAX_PAYLOAD {
        return Err(AsdError::OversizedPayload);
    }
    if (header.signature_len as usize) < ASD_MIN_SIGNATURE {
        return Err(AsdError::MissingSignature);
    }
    let expected = ASD_HEADER_LEN
        .checked_add(header.manifest_len as usize)
        .and_then(|v| v.checked_add(header.payload_len as usize))
        .and_then(|v| v.checked_add(header.signature_len as usize))
        .ok_or(AsdError::ArithmeticOverflow)?;
    if expected != bytes.len() {
        return Err(AsdError::InvalidLength);
    }
    Ok(header)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageState {
    Installed,
    Active,
    Staged,
    Failed,
    Quarantined,
}

pub const fn package_transition(from: PackageState, to: PackageState) -> bool {
    matches!((from, to),
        (PackageState::Installed, PackageState::Active)
            | (PackageState::Active, PackageState::Staged)
            | (PackageState::Staged, PackageState::Active)
            | (PackageState::Staged, PackageState::Failed)
            | (PackageState::Failed, PackageState::Staged)
            | (PackageState::Failed, PackageState::Quarantined))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(manifest: u32, payload: u32, sig: u16) -> Vec<u8> {
        let mut b = Vec::with_capacity(ASD_HEADER_LEN + manifest as usize + payload as usize + sig as usize);
        b.extend_from_slice(&ASD_MAGIC);
        b.extend_from_slice(&ASD_VERSION.to_le_bytes());
        b.extend_from_slice(&0x8664u16.to_le_bytes());
        b.extend_from_slice(&manifest.to_le_bytes());
        b.extend_from_slice(&payload.to_le_bytes());
        b.extend_from_slice(&sig.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&0u16.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());
        b.extend(core::iter::repeat_n(0u8, manifest as usize + payload as usize + sig as usize));
        b
    }

    #[test]
    fn validates_canonical_header_and_bounds() {
        assert!(validate_asd(&header(8, 32, 64)).is_ok());
        assert_eq!(validate_asd(&header(8, 32, 63)), Err(AsdError::MissingSignature));
    }

    #[test]
    fn lifecycle_is_fail_closed() {
        assert!(package_transition(PackageState::Staged, PackageState::Active));
        assert!(!package_transition(PackageState::Installed, PackageState::Staged));
    }
}
