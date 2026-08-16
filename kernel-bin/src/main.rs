#![no_std]
#![no_main]

use core::arch::global_asm;
use awe_boot_protocol::{Architecture, BootInfo};
use aweos_kernel::entry::{kernel_entry, KernelBootStatus};

global_asm!(r#"
.section .text.boot
.global _start
.type _start, @function
_start:
    cli
    mov $stack_top, %rsp
    and $-16, %rsp
    xor %rbp, %rbp
    call rust_main
1:
    hlt
    jmp 1b
.size _start, .-_start

.section .bss.stack,"aw",@nobits
.align 16
stack_bottom:
.skip 65536
stack_top:
"#);

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    serial_init();
    serial_write(b"AWEOS CellKernel\r\n");
    serial_write(b"AWEOS boot: x86_64 kernel entry\r\n");

    let info = BootInfo::empty(Architecture::X86_64);
    match kernel_entry(&info) {
        KernelBootStatus::Ready => serial_write(b"AWEOS: boot protocol validated\r\n"),
        _ => serial_write(b"AWEOS: boot protocol validation FAILED\r\n"),
    }

    serial_write(b"AWEOS: kernel is alive\r\n");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

fn serial_init() {
    unsafe {
        core::arch::asm!(
            "mov dx, 0x3F8 + 1",
            "mov al, 0",
            "out dx, al",
            "mov dx, 0x3F8 + 3",
            "mov al, 0x80",
            "out dx, al",
            "mov dx, 0x3F8",
            "mov al, 3",
            "out dx, al",
            "mov dx, 0x3F8 + 3",
            "mov al, 3",
            "out dx, al",
            "mov dx, 0x3F8 + 2",
            "mov al, 0xC7",
            "out dx, al",
            "mov dx, 0x3F8 + 4",
            "mov al, 0x0B",
            "out dx, al",
            out("al") _, out("dx") _, options(nostack, preserves_flags)
        );
    }
}

fn serial_write(bytes: &[u8]) {
    for &byte in bytes {
        unsafe {
            core::arch::asm!(
                "1:",
                "in al, dx",
                "test al, 0x20",
                "jz 1b",
                "mov dx, 0x3F8",
                "mov al, {value}",
                "out dx, al",
                value = in(reg_byte) byte,
                out("al") _,
                out("dx") _,
                options(nostack, preserves_flags)
            );
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    serial_write(b"AWEOS: KERNEL PANIC\r\n");
    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack, preserves_flags)); }
    }
}
