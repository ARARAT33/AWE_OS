#![no_std]

use super::context_switch::SwitchFrame;

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(include_str!("x86_64_switch.S"));

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackendError {
    InvalidCurrent,
    InvalidNext,
    IdenticalTarget,
}

#[cfg(target_arch = "x86_64")]
unsafe extern "C" {
    pub fn awe_x86_64_switch(current: *mut SwitchFrame, next: *const SwitchFrame);
}

pub fn validate_backend_frames(current: &SwitchFrame, next: &SwitchFrame) -> Result<(), BackendError> {
    if !current.is_well_formed() {
        return Err(BackendError::InvalidCurrent);
    }
    if !next.is_well_formed() {
        return Err(BackendError::InvalidNext);
    }
    if current.rip == next.rip && current.rsp == next.rsp {
        return Err(BackendError::IdenticalTarget);
    }
    Ok(())
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn switch_checked(current: &mut SwitchFrame, next: &SwitchFrame) -> Result<(), BackendError> {
    validate_backend_frames(current, next)?;
    // SAFETY: validation rejects malformed switch frames before entering the assembly backend.
    unsafe { awe_x86_64_switch(current as *mut _, next as *const _) }
    Ok(())
}
