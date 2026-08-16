#![no_std]

use core::arch::global_asm;
use core::sync::atomic::{AtomicU64, Ordering};

static TIMER_IRQ_COUNT: AtomicU64 = AtomicU64::new(0);

global_asm!(r#"
.section .text
.global awe_isr_timer
.type awe_isr_timer, @function
awe_isr_timer:
    cld
    pushq %rax
    pushq %rcx
    pushq %rdx
    pushq %rsi
    pushq %rdi
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    movq %rsp, %rdi
    call awe_timer_interrupt
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rdi
    popq %rsi
    popq %rdx
    popq %rcx
    popq %rax
    iretq
.size awe_isr_timer, .-awe_isr_timer
"#);

unsafe extern "C" { pub fn awe_isr_timer(); }

pub extern "C" fn awe_timer_interrupt(_saved_registers: *mut u64) {
    TIMER_IRQ_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn timer_irq_count() -> u64 { TIMER_IRQ_COUNT.load(Ordering::Acquire) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn irq_counter_is_monotonic() {
        let before = timer_irq_count();
        awe_timer_interrupt(core::ptr::null_mut());
        assert_eq!(timer_irq_count(), before + 1);
    }
}
