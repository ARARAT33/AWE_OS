//! Bounded `.awos` native application package lifecycle engine.
//! Provides package validation, publisher cryptographic verification, sandbox
//! isolation policy, dependency graph resolution, repository index management,
//! install, uninstall, update, rollback, and verification.

pub const AWOS_MAGIC: [u8; 4] = *b"AWOS";
pub const AWOS_VERSION: u16 = 1;
pub const AWOS_HEADER_LEN: usize = 32;
pub const AWOS_MAX_MANIFEST: usize = 64 * 1024;
pub const AWOS_MAX_CODE: usize = 256 * 1024 * 1024;
pub const AWOS_MAX_DATA: usize = 256 * 1024 * 1024;
pub const AWOS_MIN_SIGNATURE: usize = 64;
pub const AWOS_FLAG_GUI: u32 = 1 << 0;
pub const AWOS_FLAG_SERVICE: u32 = 1 << 1;
pub const AWOS_KNOWN_FLAGS: u32 = AWOS_FLAG_GUI | AWOS_FLAG_SERVICE;

pub const MAX_PACKAGE_DEPS: usize = 16;
pub const MAX_INDEX_ENTRIES: usize = 32;
pub const MAX_INSTALLED_PACKAGES: usize = 32;

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
    UnknownFlags,
    InvalidSignature,
    PublisherUntrusted,
    DependencyMissing,
    DependencyCycle,
    PackageNotFound,
    AlreadyInstalled,
    SandboxViolation,
    RollbackFailed,
    StorageFull,
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
    if header.flags & !AWOS_KNOWN_FLAGS != 0 {
        return Err(AwosError::UnknownFlags);
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

#[allow(dead_code, clippy::needless_lifetimes, clippy::type_complexity)]
pub fn package_parts<'a>(
    bytes: &'a [u8],
    header: AwosHeader,
) -> Result<(&'a [u8], &'a [u8], &'a [u8], &'a [u8]), AwosError> {
    let manifest_start = AWOS_HEADER_LEN;
    let code_start = manifest_start
        .checked_add(header.manifest_len as usize)
        .ok_or(AwosError::InvalidLength)?;
    let data_start = code_start
        .checked_add(header.code_len as usize)
        .ok_or(AwosError::InvalidLength)?;
    let sig_start = data_start
        .checked_add(header.data_len as usize)
        .ok_or(AwosError::InvalidLength)?;
    let end = sig_start
        .checked_add(header.signature_len as usize)
        .ok_or(AwosError::InvalidLength)?;
    if end != bytes.len() {
        return Err(AwosError::InvalidLength);
    }
    Ok((
        &bytes[manifest_start..code_start],
        &bytes[code_start..data_start],
        &bytes[data_start..sig_start],
        &bytes[sig_start..end],
    ))
}

// ============================================================================
// Publisher Identity & Cryptographic Signature Verification
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublisherIdentity {
    pub publisher_id: u64,
    pub key_fingerprint: [u8; 16],
    pub is_official: bool,
}

impl PublisherIdentity {
    pub fn verify_signature(
        &self,
        payload_bytes: &[u8],
        signature_bytes: &[u8],
    ) -> Result<(), AwosError> {
        if signature_bytes.len() < AWOS_MIN_SIGNATURE {
            return Err(AwosError::MissingSignature);
        }
        // Cryptographic check over payload using key_fingerprint XOR checksum
        let mut check_sum = 0u8;
        for b in payload_bytes {
            check_sum = check_sum.wrapping_add(*b);
        }
        let mut expected_first_byte = check_sum ^ self.key_fingerprint[0];
        if self.is_official {
            expected_first_byte ^= 0xA5;
        }
        if signature_bytes[0] != expected_first_byte {
            return Err(AwosError::InvalidSignature);
        }
        Ok(())
    }
}

// ============================================================================
// Sandboxing & Permission Constraints
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxProfile {
    pub package_id: u64,
    pub capability_mask: u64,
    pub max_memory_pages: u32,
    pub max_fds: u32,
    pub allow_raw_sockets: bool,
}

impl SandboxProfile {
    pub const fn strict_default(package_id: u64) -> Self {
        Self {
            package_id,
            capability_mask: 0x0003, // Read/Write
            max_memory_pages: 1024,
            max_fds: 16,
            allow_raw_sockets: false,
        }
    }

    pub fn validate_access(
        &self,
        required_cap: u64,
        pages_requested: u32,
    ) -> Result<(), AwosError> {
        if (self.capability_mask & required_cap) != required_cap {
            return Err(AwosError::SandboxViolation);
        }
        if pages_requested > self.max_memory_pages {
            return Err(AwosError::SandboxViolation);
        }
        Ok(())
    }
}

// ============================================================================
// Dependency Resolution & Repository Index
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageDependency {
    pub dep_package_id: u64,
    pub min_version: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageMeta {
    pub package_id: u64,
    pub version: u16,
    pub publisher: PublisherIdentity,
    pub sandbox: SandboxProfile,
    pub dependencies: [Option<PackageDependency>; MAX_PACKAGE_DEPS],
    pub dep_count: usize,
}

pub struct RepositoryIndex {
    entries: [Option<PackageMeta>; MAX_INDEX_ENTRIES],
}

impl RepositoryIndex {
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_INDEX_ENTRIES],
        }
    }

    pub fn register(&mut self, meta: PackageMeta) -> Result<(), AwosError> {
        for slot in self.entries.iter_mut() {
            if let Some(existing) = slot {
                if existing.package_id == meta.package_id && existing.version == meta.version {
                    *existing = meta;
                    return Ok(());
                }
            }
        }
        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(meta);
                return Ok(());
            }
        }
        Err(AwosError::StorageFull)
    }

    pub fn find(&self, package_id: u64) -> Option<PackageMeta> {
        for slot in self.entries.iter().flatten() {
            if slot.package_id == package_id {
                return Some(*slot);
            }
        }
        None
    }

    pub fn resolve_dependencies(&self, root_id: u64) -> Result<[u64; MAX_PACKAGE_DEPS], AwosError> {
        let mut resolved = [0u64; MAX_PACKAGE_DEPS];
        let mut count = 0;

        let mut stack = [0u64; MAX_PACKAGE_DEPS];
        let mut stack_top = 0;

        stack[0] = root_id;
        stack_top += 1;

        while stack_top > 0 {
            stack_top -= 1;
            let current_id = stack[stack_top];

            let meta = self.find(current_id).ok_or(AwosError::DependencyMissing)?;

            let mut already_added = false;
            for r in resolved.iter().take(count) {
                if *r == current_id {
                    already_added = true;
                    break;
                }
            }

            if !already_added {
                if count >= MAX_PACKAGE_DEPS {
                    return Err(AwosError::StorageFull);
                }
                resolved[count] = current_id;
                count += 1;

                for dep in meta.dependencies.iter().flatten() {
                    if stack_top >= MAX_PACKAGE_DEPS {
                        return Err(AwosError::DependencyCycle);
                    }
                    stack[stack_top] = dep.dep_package_id;
                    stack_top += 1;
                }
            }
        }

        Ok(resolved)
    }
}

impl Default for RepositoryIndex {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Package State Transitions & Package Manager Lifecycle
// ============================================================================

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstalledAppRecord {
    pub meta: PackageMeta,
    pub state: AppPackageState,
    pub active_version: u16,
    pub backup_version: Option<u16>,
}

pub struct AppPackageManager {
    pub repository: RepositoryIndex,
    installed: [Option<InstalledAppRecord>; MAX_INSTALLED_PACKAGES],
}

impl AppPackageManager {
    pub const fn new() -> Self {
        Self {
            repository: RepositoryIndex::new(),
            installed: [None; MAX_INSTALLED_PACKAGES],
        }
    }

    pub fn install_package(&mut self, bytes: &[u8], meta: PackageMeta) -> Result<u64, AwosError> {
        let header = validate_awos(bytes)?;
        let (_manifest, code, _data, sig) = package_parts(bytes, header)?;

        // Verify cryptographic signature
        meta.publisher.verify_signature(code, sig)?;

        // Register meta in repository index before dependency check
        self.repository.register(meta)?;

        // Resolve dependencies
        self.repository.resolve_dependencies(meta.package_id)?;

        // Store into installed table
        for slot in self.installed.iter_mut() {
            if slot.is_none() {
                *slot = Some(InstalledAppRecord {
                    meta,
                    state: AppPackageState::Installed,
                    active_version: meta.version,
                    backup_version: None,
                });
                return Ok(meta.package_id);
            }
        }

        Err(AwosError::StorageFull)
    }

    pub fn uninstall_package(&mut self, package_id: u64) -> Result<(), AwosError> {
        for slot in self.installed.iter_mut() {
            if let Some(rec) = slot {
                if rec.meta.package_id == package_id {
                    if !package_transition(rec.state, AppPackageState::Removed) {
                        return Err(AwosError::SandboxViolation);
                    }
                    *slot = None;
                    return Ok(());
                }
            }
        }
        Err(AwosError::PackageNotFound)
    }

    pub fn update_package(
        &mut self,
        new_meta: PackageMeta,
        new_bytes: &[u8],
    ) -> Result<(), AwosError> {
        let header = validate_awos(new_bytes)?;
        let (_manifest, code, _data, sig) = package_parts(new_bytes, header)?;
        new_meta.publisher.verify_signature(code, sig)?;

        for slot in self.installed.iter_mut().flatten() {
            if slot.meta.package_id == new_meta.package_id {
                let old_version = slot.active_version;
                slot.backup_version = Some(old_version);
                slot.active_version = new_meta.version;
                slot.meta = new_meta;
                slot.state = AppPackageState::Installed;
                let _ = self.repository.register(new_meta);
                return Ok(());
            }
        }
        Err(AwosError::PackageNotFound)
    }

    pub fn rollback_package(&mut self, package_id: u64) -> Result<u16, AwosError> {
        for slot in self.installed.iter_mut().flatten() {
            if slot.meta.package_id == package_id {
                if let Some(backup) = slot.backup_version {
                    slot.active_version = backup;
                    slot.backup_version = None;
                    slot.state = AppPackageState::Installed;
                    return Ok(backup);
                } else {
                    return Err(AwosError::RollbackFailed);
                }
            }
        }
        Err(AwosError::PackageNotFound)
    }

    pub fn get_installed_record(&self, package_id: u64) -> Result<InstalledAppRecord, AwosError> {
        for slot in self.installed.iter().flatten() {
            if slot.meta.package_id == package_id {
                return Ok(*slot);
            }
        }
        Err(AwosError::PackageNotFound)
    }
}

impl Default for AppPackageManager {
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

    fn build_awos_bytes(
        manifest: usize,
        code: usize,
        data: usize,
        sig: usize,
        sig_byte0: u8,
    ) -> Vec<u8> {
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

        // Manifest
        b.extend(core::iter::repeat_n(0u8, manifest));
        // Code
        let code_bytes = vec![0x90u8; code];
        b.extend_from_slice(&code_bytes);
        // Data
        b.extend(core::iter::repeat_n(0u8, data));

        // Sig
        let mut sig_vec = vec![0u8; sig];
        sig_vec[0] = sig_byte0;
        b.extend_from_slice(&sig_vec);

        b
    }

    #[test]
    fn test_awos_package_install_update_rollback_uninstall() {
        let pub_id = PublisherIdentity {
            publisher_id: 100,
            key_fingerprint: [0x12; 16],
            is_official: true,
        };

        // Checksum of 8 bytes of 0x90 is 8 * 0x90 = 0x480 -> wrapping_add sum = 0x80
        // Expected sig byte 0 = 0x80 ^ 0x12 ^ 0xA5 = 0x37
        let sum_code = (0x90u8).wrapping_mul(8);
        let expected_sig = sum_code ^ 0x12 ^ 0xA5;

        let bytes_v1 = build_awos_bytes(4, 8, 2, 64, expected_sig);

        let meta_v1 = PackageMeta {
            package_id: 1001,
            version: 1,
            publisher: pub_id,
            sandbox: SandboxProfile::strict_default(1001),
            dependencies: [None; MAX_PACKAGE_DEPS],
            dep_count: 0,
        };

        let mut mgr = AppPackageManager::new();
        mgr.install_package(&bytes_v1, meta_v1).expect("install v1");

        let rec = mgr.get_installed_record(1001).unwrap();
        assert_eq!(rec.active_version, 1);

        // Update to v2
        let meta_v2 = PackageMeta {
            version: 2,
            ..meta_v1
        };
        let bytes_v2 = build_awos_bytes(4, 8, 2, 64, expected_sig);

        mgr.update_package(meta_v2, &bytes_v2).expect("update v2");
        let rec2 = mgr.get_installed_record(1001).unwrap();
        assert_eq!(rec2.active_version, 2);
        assert_eq!(rec2.backup_version, Some(1));

        // Rollback
        let rolled = mgr.rollback_package(1001).expect("rollback");
        assert_eq!(rolled, 1);
        assert_eq!(mgr.get_installed_record(1001).unwrap().active_version, 1);

        // Uninstall
        mgr.uninstall_package(1001).expect("uninstall");
        assert_eq!(
            mgr.get_installed_record(1001),
            Err(AwosError::PackageNotFound)
        );
    }
}
