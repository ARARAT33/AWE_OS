#![no_std]

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(include_str!("x86_64_switch.S"));

use super::context_switch::SwitchFrame;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackendError { InvalidCurrent, InvalidNext }

#[cfg(target_arch = "x86_64")]
unsafe extern "C" {
    pub fn awe_x86_64_switch(current: *mut SwitchFrame, next: *const SwitchFrame);
}

pub fn validate_backend_frames(current: &SwitchFrame, next: &SwitchFrame) -> Result<(), BackendError> {
    if !current.is_well_formed() { return Err(BackendError::InvalidCurrent); }
    if !next.is_well_formed() { return Err(BackendError::InvalidNext); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(rip: u64, rsp: u64) -> SwitchFrame { SwitchFrame { rip, rsp, ..SwitchFrame::default() } }
    #[test] fn valid_frames_pass_backend_gate() {
        assert!(validate_backend_frames(&frame(0x400000,0x800000), &frame(0x500000,0x900000)).is_ok());
    }
    #[test] fn invalid_current_is_rejected() {
        assert_eq!(validate_backend_frames(&frame(0,0x800000), &frame(0x500000,0x900000)), Err(BackendError::InvalidCurrent));
    }
    #[test] fn invalid_next_is_rejected() {
        assert_eq!(validate_backend_frames(&frame(0x400000,0x800000), &frame(0,0x900000)), Err(BackendError::InvalidNext));
    }
}
