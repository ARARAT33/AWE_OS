#![no_std]

use super::isr::{InterruptFrame, is_exception_with_error_code};

/// Common Rust-side interrupt entry point. Assembly stubs are responsible for
/// constructing the frame; this function validates it before dispatching any
/// architecture-independent policy.
#[inline(never)]
pub extern "C" fn interrupt_entry(frame: &mut InterruptFrame) -> InterruptAction {
    if !frame.validate() {
        return InterruptAction::Fatal;
    }
    let vector = frame.vector as u8;
    if is_exception_with_error_code(vector) {
        InterruptAction::Exception
    } else if vector == 32 {
        InterruptAction::Timer
    } else {
        InterruptAction::Unhandled
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterruptAction {
    Timer = 0,
    Exception = 1,
    Unhandled = 2,
    Fatal = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn valid(vector: u64) -> InterruptFrame {
        let mut f = InterruptFrame::empty();
        f.rip = 0x400000;
        f.rsp = 0x800000;
        f.rflags = 0x202;
        f.vector = vector;
        f
    }
    #[test]
    fn timer_is_classified() {
        let mut f = valid(32);
        assert_eq!(interrupt_entry(&mut f), InterruptAction::Timer);
    }
    #[test]
    fn page_fault_is_classified() {
        let mut f = valid(14);
        assert_eq!(interrupt_entry(&mut f), InterruptAction::Exception);
    }
    #[test]
    fn invalid_frame_fails_closed() {
        let mut f = InterruptFrame::empty();
        f.vector = 32;
        assert_eq!(interrupt_entry(&mut f), InterruptAction::Fatal);
    }
}
