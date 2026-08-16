#![no_std]

use core::arch::global_asm;

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

extern "C" { pub fn awe_isr_timer(); }

#[no_mangle]
pub extern "C" fn awe_timer_interrupt(_saved_registers: *mut u64) {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}
