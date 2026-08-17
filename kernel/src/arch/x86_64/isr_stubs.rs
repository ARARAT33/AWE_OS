#![no_std]
use core::arch::global_asm;
use core::sync::atomic::{AtomicU64, Ordering};
static TIMER_IRQ_COUNT: AtomicU64 = AtomicU64::new(0);
global_asm!(
    r#".intel_syntax noprefix
.section .text
.global awe_isr_timer
.type awe_isr_timer, @function
awe_isr_timer:
    cld
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    mov rdi, rsp
    call awe_timer_interrupt
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax
    iretq
.size awe_isr_timer, .-awe_isr_timer
.att_syntax prefix
"#
);
unsafe extern "C" {
    pub fn awe_isr_timer();
}
pub extern "C" fn awe_timer_interrupt(_saved_registers: *mut u64) {
    TIMER_IRQ_COUNT.fetch_add(1, Ordering::Relaxed);
}
pub fn timer_irq_count() -> u64 {
    TIMER_IRQ_COUNT.load(Ordering::Acquire)
}
