#![no_std]

use super::context_switch::{SwitchError, SwitchFrame};

/// Validated hand-off contract between scheduler and architecture backend.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SwitchPlan { pub current: SwitchFrame, pub next: SwitchFrame }

impl SwitchPlan {
    pub const fn new(current: SwitchFrame, next: SwitchFrame) -> Result<Self, SwitchError> {
        if !current.is_well_formed() { return Err(SwitchError::InvalidCurrent); }
        if !next.is_well_formed() { return Err(SwitchError::InvalidNext); }
        Ok(Self { current, next })
    }
    pub const fn next_rip(&self) -> u64 { self.next.rip }
    pub const fn next_rsp(&self) -> u64 { self.next.rsp }
    pub const fn target(&self) -> &SwitchFrame { &self.next }
}

pub fn build_plan(current: &SwitchFrame, next: &SwitchFrame) -> Result<SwitchPlan, SwitchError> {
    SwitchPlan::new(*current, *next)
}

/// Rejects an accidental no-op before the architecture backend is entered.
pub fn validate_transition(plan: &SwitchPlan) -> Result<(), SwitchError> {
    if plan.current.rip == plan.next.rip && plan.current.rsp == plan.next.rsp {
        return Err(SwitchError::InvalidNext);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(rip: u64, rsp: u64) -> SwitchFrame { SwitchFrame { rip, rsp, ..SwitchFrame::default() } }
    #[test] fn plan_contains_target() {
        let p = build_plan(&frame(0x400000,0x800000), &frame(0x500000,0x900000)).unwrap();
        assert_eq!(p.next_rip(),0x500000); assert_eq!(p.next_rsp(),0x900000); assert_eq!(p.target().rip,0x500000);
    }
    #[test] fn invalid_frames_rejected() {
        assert_eq!(build_plan(&frame(0,0x800000),&frame(0x500000,0x900000)),Err(SwitchError::InvalidCurrent));
        assert_eq!(build_plan(&frame(0x400000,0x800000),&frame(0,0x900000)),Err(SwitchError::InvalidNext));
    }
    #[test] fn identical_transition_rejected() {
        let p=build_plan(&frame(0x400000,0x800000),&frame(0x400000,0x800000)).unwrap();
        assert_eq!(validate_transition(&p),Err(SwitchError::InvalidNext));
    }
}
