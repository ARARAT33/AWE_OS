#![no_std]

use super::context::CpuContext;

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct SwitchFrame {
    pub rbx: u64, pub rbp: u64, pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    pub rsp: u64, pub rip: u64,
}

impl SwitchFrame {
    pub const fn from_context(ctx: &CpuContext) -> Self {
        Self { rbx: ctx.rbx, rbp: ctx.rbp, r12: ctx.r12, r13: ctx.r13, r14: ctx.r14, r15: ctx.r15, rsp: ctx.rsp, rip: ctx.rip }
    }
    pub fn apply_to_context(&self, ctx: &mut CpuContext) {
        ctx.rbx=self.rbx; ctx.rbp=self.rbp; ctx.r12=self.r12; ctx.r13=self.r13;
        ctx.r14=self.r14; ctx.r15=self.r15; ctx.rsp=self.rsp; ctx.rip=self.rip;
    }
    pub const fn is_well_formed(&self) -> bool {
        self.rip != 0 && self.rsp != 0 && self.rip >> 48 == 0 && self.rsp >> 48 == 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwitchError { InvalidCurrent, InvalidNext }

pub fn prepare_switch(current: &CpuContext, next: &CpuContext) -> Result<(SwitchFrame, SwitchFrame), SwitchError> {
    let old=SwitchFrame::from_context(current); let new=SwitchFrame::from_context(next);
    if !old.is_well_formed() { return Err(SwitchError::InvalidCurrent); }
    if !new.is_well_formed() { return Err(SwitchError::InvalidNext); }
    Ok((old,new))
}

/// Architecture boundary. The current implementation is a safe, non-switching
/// validation stub; the actual register-changing assembly is enabled only when
/// the x86_64 target integration is present.
#[inline(never)]
pub unsafe fn context_switch(current: *mut SwitchFrame, next: *const SwitchFrame) -> Result<(), SwitchError> {
    if current.is_null() { return Err(SwitchError::InvalidCurrent); }
    if next.is_null() { return Err(SwitchError::InvalidNext); }
    let old = &*current;
    let new = &*next;
    if !old.is_well_formed() { return Err(SwitchError::InvalidCurrent); }
    if !new.is_well_formed() { return Err(SwitchError::InvalidNext); }
    // A real switch must be implemented in target-specific assembly and must
    // restore RIP/RSP as an atomic ABI operation. Never emulate it in Rust.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn frame_round_trip() {
        let source=CpuContext{rip:0x400000,rsp:0x800000,rflags:2,cr3:0x100000,rbx:1,rbp:2,r12:3,r13:4,r14:5,r15:6};
        let frame=SwitchFrame::from_context(&source); let mut restored=CpuContext::kernel_entry(0x400000,0x800000,0x100000);
        frame.apply_to_context(&mut restored); assert_eq!(frame,SwitchFrame::from_context(&restored));
    }
    #[test] fn invalid_state_rejected() {
        let invalid=CpuContext::default(); let valid=CpuContext::kernel_entry(0x400000,0x800000,0x100000);
        assert_eq!(prepare_switch(&invalid,&valid),Err(SwitchError::InvalidCurrent));
        assert_eq!(prepare_switch(&valid,&invalid),Err(SwitchError::InvalidNext));
    }
    #[test] fn null_pointer_rejected() {
        let valid=SwitchFrame::from_context(&CpuContext::kernel_entry(0x400000,0x800000,0x100000));
        assert_eq!(unsafe{context_switch(core::ptr::null_mut(),&valid)},Err(SwitchError::InvalidCurrent));
        assert_eq!(unsafe{context_switch(&mut SwitchFrame::default(),core::ptr::null())},Err(SwitchError::InvalidNext));
    }
}
