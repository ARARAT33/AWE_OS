.section .text
.global awe_x86_64_switch
.type awe_x86_64_switch,@function

# void awe_x86_64_switch(SwitchFrame *current, const SwitchFrame *next)
# System V AMD64 ABI: RDI=current, RSI=next.
# Frame: rbx, rbp, r12, r13, r14, r15, rsp, rip.
# This backend changes only the state represented by SwitchFrame.
# CR3, RFLAGS, segments and interrupt state remain architecture-layer policy.

awe_x86_64_switch:
    test %rdi, %rdi
    jz .Linvalid
    test %rsi, %rsi
    jz .Linvalid

    mov %rbx, 0(%rdi)
    mov %rbp, 8(%rdi)
    mov %r12, 16(%rdi)
    mov %r13, 24(%rdi)
    mov %r14, 32(%rdi)
    mov %r15, 40(%rdi)
    mov %rsp, 48(%rdi)
    lea .Lresume(%rip), %rax
    mov %rax, 56(%rdi)

    mov 0(%rsi), %rbx
    mov 8(%rsi), %rbp
    mov 16(%rsi), %r12
    mov 24(%rsi), %r13
    mov 32(%rsi), %r14
    mov 40(%rsi), %r15
    mov 48(%rsi), %rsp
    mov 56(%rsi), %rax
    jmp *%rax

.Lresume:
    ret

.Linvalid:
    ud2

.size awe_x86_64_switch, .-awe_x86_64_switch
