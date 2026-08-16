#![no_std]
#![no_main]

use core::arch::global_asm;
use awe_boot_protocol::{Architecture, BootInfo};
use aweos_kernel::entry::{kernel_entry, KernelBootStatus};

#[used]
#[unsafe(link_section = ".multiboot2_header")]
#[unsafe(no_mangle)]
static MULTIBOOT2_HEADER: [u32; 4] = [0xE85250D6, 0, 16, 0x17ADA8E8];

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
            "mov dx, 0x3F9", "xor al, al", "out dx, al",
            "mov dx, 0x3FB", "mov al, 0x80", "out dx, al",
            "mov dx, 0x3F8", "mov al, 3", "out dx, al",
            "mov dx, 0x3FB", "mov al, 3", "out dx, al",
            "mov dx, 0x3FA", "mov al, 0xC7", "out dx, al",
            "mov dx, 0x3FC", "mov al, 0x0B", "out dx, al",
            out("al") _, out("dx") _, options(nostack, preserves_flags)
        );
    }
}

fn serial_write(bytes: &[u8]) {
    for &byte in bytes {
        unsafe {
            core::arch::asm!(
                "mov dx, 0x3FD",
                "1: in al, dx",
                "test al, 0x20",
                "jz 1b",
                "mov dx, 0x3F8",
                "mov al, {byte}",
                "out dx, al",
                byte = in(reg) u32::from(byte),
                out("al") _, out("dx") _,
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
