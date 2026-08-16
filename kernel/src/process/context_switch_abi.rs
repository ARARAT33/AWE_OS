#![no_std]

use super::context_switch::{SwitchError, SwitchFrame};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SwitchPlan {
    pub current: SwitchFrame,
    pub next: SwitchFrame,
}

impl SwitchPlan {
    pub const fn new(current: SwitchFrame, next: SwitchFrame) -> Result<Self, SwitchError> {
        if !current.is_well_formed() { return Err(SwitchError::InvalidCurrent); }
        if !next.is_well_formed() { return Err(SwitchError::InvalidNext); }
        Ok(Self { current, next })
    }
    pub const fn next_rip(&self) -> u64 { self.next.rip }
    pub const fn next_rsp(&self) -> u64 { self.next.rsp }
}

pub fn build_plan(current: &SwitchFrame, next: &SwitchFrame) -> Result<SwitchPlan, SwitchError> {
    SwitchPlan::new(*current, *next)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(rip: u64, rsp: u64) -> SwitchFrame { SwitchFrame { rip, rsp, ..SwitchFrame::default() } }
    #[test]
    fn plan_contains_exact_target_state() {
        let plan = build_plan(&frame(0x400000, 0x800000), &frame(0x500000, 0x900000)).unwrap();
        assert_eq!(plan.next_rip(), 0x500000); assert_eq!(plan.next_rsp(), 0x900000);
    }
    #[test]
    fn invalid_frames_never_form_a_plan() {
        assert_eq!(build_plan(&frame(0, 0x800000), &frame(0x500000, 0x900000)), Err(SwitchError::InvalidCurrent));
        assert_eq!(build_plan(&frame(0x400000, 0x800000), &frame(0, 0x900000)), Err(SwitchError::InvalidNext));
    }
}
