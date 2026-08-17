#![no_std]

//! AWE native application admission service. Package policy remains outside
//! CellKernel and every untrusted field is validated before admission.

pub const AWE_APP_ABI_MAJOR: u16 = 1;
pub const AWE_APP_ABI_MINOR: u16 = 3;
pub const MAX_DEPS: usize = 32;
pub const MAX_RESOURCES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState { Installed, Starting, Running, Stopped, Failed, Quarantined }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppManifest {
    pub id: AppId,
    pub abi_major: u16,
    pub abi_minor: u16,
    pub memory_limit_pages: u32,
    pub capability_mask: u64,
    pub dependency_count: u16,
    pub resource_count: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppError { InvalidManifest, AlreadyRunning, NotFound, CapabilityDenied, TooManyDependencies, TooManyResources }

pub fn validate_manifest(manifest: AppManifest) -> Result<(), AppError> {
    if manifest.id.0 == 0 || manifest.abi_major != AWE_APP_ABI_MAJOR
        || manifest.abi_minor > AWE_APP_ABI_MINOR
        || manifest.memory_limit_pages == 0
    { return Err(AppError::InvalidManifest); }
    if manifest.dependency_count as usize > MAX_DEPS { return Err(AppError::TooManyDependencies); }
    if manifest.resource_count as usize > MAX_RESOURCES { return Err(AppError::TooManyResources); }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageHeader {
    pub version: u16,
    pub manifest_len: u32,
    pub payload_len: u32,
    pub signature_len: u16,
}

pub const PACKAGE_HEADER_SIZE: usize = 14;
pub const MAX_MANIFEST: usize = 64 * 1024;
pub const MAX_PAYLOAD: usize = 64 * 1024 * 1024;
pub const MIN_SIGNATURE: usize = 32;

pub fn validate_package(header: PackageHeader, total_len: usize) -> Result<(), AppError> {
    if header.version == 0 || header.manifest_len as usize > MAX_MANIFEST
        || header.payload_len as usize > MAX_PAYLOAD
        || header.signature_len as usize < MIN_SIGNATURE { return Err(AppError::InvalidManifest); }
    let expected = PACKAGE_HEADER_SIZE
        .checked_add(header.manifest_len as usize)
        .and_then(|v| v.checked_add(header.payload_len as usize))
        .and_then(|v| v.checked_add(header.signature_len as usize))
        .ok_or(AppError::InvalidManifest)?;
    if expected != total_len { return Err(AppError::InvalidManifest); }
    Ok(())
}

/// Validates that a dependency/resource declaration can be represented by the
/// bounded manifest counters without allowing arithmetic overflow.
pub fn validate_declarations(dependencies: usize, resources: usize) -> Result<(), AppError> {
    if dependencies > MAX_DEPS { return Err(AppError::TooManyDependencies); }
    if resources > MAX_RESOURCES { return Err(AppError::TooManyResources); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_oversized_package() {
        let h = PackageHeader { version: 1, manifest_len: 1, payload_len: (MAX_PAYLOAD + 1) as u32, signature_len: 64 };
        assert_eq!(validate_package(h, 0), Err(AppError::InvalidManifest));
    }
    #[test]
    fn rejects_short_signature() {
        let h = PackageHeader { version: 1, manifest_len: 1, payload_len: 1, signature_len: 31 };
        assert_eq!(validate_package(h, PACKAGE_HEADER_SIZE + 1 + 1 + 31), Err(AppError::InvalidManifest));
    }
    #[test]
    fn validates_bounded_manifest() {
        let m = AppManifest { id: AppId(1), abi_major: 1, abi_minor: 3, memory_limit_pages: 4, capability_mask: 1, dependency_count: 1, resource_count: 1 };
        assert!(validate_manifest(m).is_ok());
    }
    #[test]
    fn declaration_limits_fail_closed() {
        assert_eq!(validate_declarations(MAX_DEPS + 1, 0), Err(AppError::TooManyDependencies));
        assert_eq!(validate_declarations(0, MAX_RESOURCES + 1), Err(AppError::TooManyResources));
    }
}
