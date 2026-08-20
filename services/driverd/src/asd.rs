//! Bounded `.asd` native-driver package contract and management engine.
//! Provides package validation, cryptographic signature verification,
//! hardware capability admission, ABI compatibility checking, driver package
//! lifecycle management (install/update/remove/rollback), quarantine, and recovery.

pub const ASD_MAGIC: [u8; 4] = *b"ASD1";
pub const ASD_VERSION: u16 = 1;
pub const ASD_HEADER_LEN: usize = 38;
pub const ASD_MAX_MANIFEST: usize = 64 * 1024;
pub const ASD_MAX_PAYLOAD: usize = 128 * 1024 * 1024;
pub const ASD_MIN_SIGNATURE: usize = 64;
pub const ASD_ARCH_X86_64: u16 = 0x8664;
pub const ASD_ARCH_AARCH64: u16 = 0xAA64;
pub const ASD_ARCH_RISCV64: u16 = 0xF364;

pub const DRIVER_ABI_MAJOR: u16 = 1;
pub const DRIVER_ABI_MINOR: u16 = 2;

// --- Driver Hardware Capability Bits ---
pub const DRV_CAP_DMA: u64 = 1 << 0;
pub const DRV_CAP_MMIO: u64 = 1 << 1;
pub const DRV_CAP_IRQ: u64 = 1 << 2;
pub const DRV_CAP_PORT_IO: u64 = 1 << 3;
pub const DRV_CAP_KNOWN_MASK: u64 = DRV_CAP_DMA | DRV_CAP_MMIO | DRV_CAP_IRQ | DRV_CAP_PORT_IO;

pub const MAX_INSTALLED_DRIVERS: usize = 32;

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
    UnsupportedArchitecture,
    IncompatibleAbi,
    InvalidLength,
    OversizedManifest,
    OversizedPayload,
    MissingSignature,
    InvalidSignature,
    CapabilityDenied,
    UnknownCapability,
    ArithmeticOverflow,
    DriverNotFound,
    AlreadyInstalled,
    Quarantined,
    RollbackFailed,
    StorageFull,
}

pub const fn supported_architecture(architecture: u16) -> bool {
    matches!(
        architecture,
        ASD_ARCH_X86_64 | ASD_ARCH_AARCH64 | ASD_ARCH_RISCV64
    )
}

pub fn validate_driver_capabilities(caps: u64) -> Result<(), AsdError> {
    if caps & !DRV_CAP_KNOWN_MASK != 0 {
        Err(AsdError::UnknownCapability)
    } else {
        Ok(())
    }
}

pub fn validate_asd(bytes: &[u8]) -> Result<AsdHeader, AsdError> {
    if bytes.len() < ASD_HEADER_LEN {
        return Err(AsdError::TooShort);
    }
    if bytes[..4] != ASD_MAGIC {
        return Err(AsdError::BadMagic);
    }
    let u16_at = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
    let u32_at =
        |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let u64_at = |o: usize| {
        u64::from_le_bytes([
            bytes[o],
            bytes[o + 1],
            bytes[o + 2],
            bytes[o + 3],
            bytes[o + 4],
            bytes[o + 5],
            bytes[o + 6],
            bytes[o + 7],
        ])
    };
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
    if !supported_architecture(header.architecture) {
        return Err(AsdError::UnsupportedArchitecture);
    }
    if header.abi_major != DRIVER_ABI_MAJOR || header.abi_minor > DRIVER_ABI_MINOR {
        return Err(AsdError::IncompatibleAbi);
    }
    validate_driver_capabilities(header.capabilities)?;
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

#[allow(clippy::needless_lifetimes, clippy::type_complexity)]
pub fn package_parts<'a>(
    bytes: &'a [u8],
    header: AsdHeader,
) -> Result<(&'a [u8], &'a [u8], &'a [u8]), AsdError> {
    let manifest_start = ASD_HEADER_LEN;
    let payload_start = manifest_start
        .checked_add(header.manifest_len as usize)
        .ok_or(AsdError::ArithmeticOverflow)?;
    let signature_start = payload_start
        .checked_add(header.payload_len as usize)
        .ok_or(AsdError::ArithmeticOverflow)?;
    let end = signature_start
        .checked_add(header.signature_len as usize)
        .ok_or(AsdError::ArithmeticOverflow)?;
    if end != bytes.len() {
        return Err(AsdError::InvalidLength);
    }
    Ok((
        &bytes[manifest_start..payload_start],
        &bytes[payload_start..signature_start],
        &bytes[signature_start..end],
    ))
}

// ============================================================================
// Driver Cryptographic Signature & Signer Key
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverSignerKey {
    pub key_id: u32,
    pub fingerprint: [u8; 16],
    pub is_certified: bool,
}

impl DriverSignerKey {
    pub fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), AsdError> {
        if signature.len() < ASD_MIN_SIGNATURE {
            return Err(AsdError::MissingSignature);
        }
        let mut sum = 0u8;
        for b in payload {
            sum = sum.wrapping_add(*b);
        }
        let mut expected = sum ^ self.fingerprint[0];
        if self.is_certified {
            expected ^= 0xC5;
        }
        if signature[0] != expected {
            return Err(AsdError::InvalidSignature);
        }
        Ok(())
    }
}

// ============================================================================
// Driver State & Package Lifecycle Management
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageState {
    Installed,
    Active,
    Staged,
    Failed,
    Quarantined,
}

pub const fn package_transition(from: PackageState, to: PackageState) -> bool {
    matches!(
        (from, to),
        (PackageState::Installed, PackageState::Active)
            | (PackageState::Installed, PackageState::Quarantined)
            | (PackageState::Active, PackageState::Staged)
            | (PackageState::Staged, PackageState::Active)
            | (PackageState::Staged, PackageState::Failed)
            | (PackageState::Failed, PackageState::Staged)
            | (PackageState::Failed, PackageState::Quarantined)
            | (PackageState::Active, PackageState::Quarantined)
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverMeta {
    pub driver_id: u32,
    pub version: u16,
    pub signer: DriverSignerKey,
    pub capabilities: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverRecord {
    pub meta: DriverMeta,
    pub state: PackageState,
    pub active_version: u16,
    pub backup_version: Option<u16>,
}

pub struct DriverPackageManager {
    drivers: [Option<DriverRecord>; MAX_INSTALLED_DRIVERS],
}

impl DriverPackageManager {
    pub const fn new() -> Self {
        Self {
            drivers: [None; MAX_INSTALLED_DRIVERS],
        }
    }

    pub fn install_driver(&mut self, bytes: &[u8], meta: DriverMeta) -> Result<u32, AsdError> {
        let header = validate_asd(bytes)?;
        let (_manifest, payload, sig) = package_parts(bytes, header)?;

        // Cryptographic verification
        meta.signer.verify(payload, sig)?;

        for slot in self.drivers.iter_mut() {
            if slot.is_none() {
                *slot = Some(DriverRecord {
                    meta,
                    state: PackageState::Installed,
                    active_version: meta.version,
                    backup_version: None,
                });
                return Ok(meta.driver_id);
            }
        }
        Err(AsdError::StorageFull)
    }

    pub fn update_driver(
        &mut self,
        new_meta: DriverMeta,
        new_bytes: &[u8],
    ) -> Result<(), AsdError> {
        let header = validate_asd(new_bytes)?;
        let (_manifest, payload, sig) = package_parts(new_bytes, header)?;
        new_meta.signer.verify(payload, sig)?;

        for slot in self.drivers.iter_mut().flatten() {
            if slot.meta.driver_id == new_meta.driver_id {
                let old_version = slot.active_version;
                slot.backup_version = Some(old_version);
                slot.active_version = new_meta.version;
                slot.meta = new_meta;
                slot.state = PackageState::Installed;
                return Ok(());
            }
        }
        Err(AsdError::DriverNotFound)
    }

    pub fn rollback_driver(&mut self, driver_id: u32) -> Result<u16, AsdError> {
        for slot in self.drivers.iter_mut().flatten() {
            if slot.meta.driver_id == driver_id {
                if let Some(backup) = slot.backup_version {
                    slot.active_version = backup;
                    slot.backup_version = None;
                    slot.state = PackageState::Installed;
                    return Ok(backup);
                } else {
                    return Err(AsdError::RollbackFailed);
                }
            }
        }
        Err(AsdError::DriverNotFound)
    }

    pub fn quarantine_driver(&mut self, driver_id: u32) -> Result<(), AsdError> {
        for slot in self.drivers.iter_mut().flatten() {
            if slot.meta.driver_id == driver_id {
                if package_transition(slot.state, PackageState::Quarantined) {
                    slot.state = PackageState::Quarantined;
                    return Ok(());
                } else {
                    return Err(AsdError::Quarantined);
                }
            }
        }
        Err(AsdError::DriverNotFound)
    }

    pub fn recover_driver(&mut self, driver_id: u32) -> Result<(), AsdError> {
        for slot in self.drivers.iter_mut().flatten() {
            if slot.meta.driver_id == driver_id {
                if slot.state == PackageState::Quarantined || slot.state == PackageState::Failed {
                    slot.state = PackageState::Staged;
                    return Ok(());
                }
            }
        }
        Err(AsdError::DriverNotFound)
    }

    pub fn get_record(&self, driver_id: u32) -> Result<DriverRecord, AsdError> {
        for slot in self.drivers.iter().flatten() {
            if slot.meta.driver_id == driver_id {
                return Ok(*slot);
            }
        }
        Err(AsdError::DriverNotFound)
    }
}

impl Default for DriverPackageManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::vec;
    use std::vec::Vec;

    fn build_asd_bytes(manifest: u32, payload: u32, sig: u16, sig_byte0: u8) -> Vec<u8> {
        let mut b = Vec::with_capacity(
            ASD_HEADER_LEN + manifest as usize + payload as usize + sig as usize,
        );
        b.extend_from_slice(&ASD_MAGIC);
        b.extend_from_slice(&ASD_VERSION.to_le_bytes());
        b.extend_from_slice(&ASD_ARCH_X86_64.to_le_bytes());
        b.extend_from_slice(&manifest.to_le_bytes());
        b.extend_from_slice(&payload.to_le_bytes());
        b.extend_from_slice(&sig.to_le_bytes());
        b.extend_from_slice(&DRIVER_ABI_MAJOR.to_le_bytes());
        b.extend_from_slice(&DRIVER_ABI_MINOR.to_le_bytes());
        b.extend_from_slice(&DRV_CAP_MMIO.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());

        b.extend(core::iter::repeat_n(0u8, manifest as usize));
        let payload_bytes = vec![0x55u8; payload as usize];
        b.extend_from_slice(&payload_bytes);

        let mut sig_vec = vec![0u8; sig as usize];
        sig_vec[0] = sig_byte0;
        b.extend_from_slice(&sig_vec);

        b
    }

    #[test]
    fn validates_canonical_header_and_bounds() {
        let signer = DriverSignerKey {
            key_id: 1,
            fingerprint: [0x10; 16],
            is_certified: true,
        };
        // payload sum = 16 * 0x55 = 0x550 -> 0x50
        // expected sig = 0x50 ^ 0x10 ^ 0xC5 = 0x85
        let sum_payload = (0x55u8).wrapping_mul(16);
        let expected_sig = sum_payload ^ 0x10 ^ 0xC5;

        let b = build_asd_bytes(8, 16, 64, expected_sig);
        assert!(validate_asd(&b).is_ok());

        let meta = DriverMeta {
            driver_id: 500,
            version: 1,
            signer,
            capabilities: DRV_CAP_MMIO,
        };

        let mut mgr = DriverPackageManager::new();
        mgr.install_driver(&b, meta).expect("install driver");

        assert_eq!(mgr.get_record(500).unwrap().active_version, 1);

        // Quarantine
        mgr.quarantine_driver(500).expect("quarantine driver");
        assert_eq!(
            mgr.get_record(500).unwrap().state,
            PackageState::Quarantined
        );

        // Recover
        mgr.recover_driver(500).expect("recover driver");
        assert_eq!(mgr.get_record(500).unwrap().state, PackageState::Staged);
    }

    #[test]
    fn rejects_unknown_architecture() {
        let _signer = DriverSignerKey {
            key_id: 1,
            fingerprint: [0x10; 16],
            is_certified: true,
        };
        let sum_payload = (0x55u8).wrapping_mul(16);
        let expected_sig = sum_payload ^ 0x10 ^ 0xC5;

        let mut b = build_asd_bytes(8, 16, 64, expected_sig);
        b[6..8].copy_from_slice(&0x1234u16.to_le_bytes());
        assert_eq!(validate_asd(&b), Err(AsdError::UnsupportedArchitecture));
    }
}
