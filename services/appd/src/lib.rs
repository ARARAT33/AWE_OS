#![no_std]

//! AWE application service. Native applications are user-space objects; this
//! service owns package admission, lifecycle and sandbox policy rather than
//! embedding application logic in CellKernel.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Installed,
    Starting,
    Running,
    Stopped,
    Failed,
    Quarantined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppManifest {
    pub id: AppId,
    pub abi_major: u16,
    pub memory_limit_pages: u32,
    pub capability_mask: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppError {
    InvalidManifest,
    AlreadyRunning,
    NotFound,
    CapabilityDenied,
}

pub const AWE_APP_ABI_MAJOR: u16 = 1;

pub fn validate_manifest(manifest: AppManifest) -> Result<(), AppError> {
    if manifest.abi_major != AWE_APP_ABI_MAJOR || manifest.memory_limit_pages == 0 {
        return Err(AppError::InvalidManifest);
    }
    Ok(())
}
