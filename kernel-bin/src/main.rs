#![no_std]
#![no_main]

use core::arch::global_asm;
use awe_boot_protocol::{Architecture, BootInfo, MemoryRegion, AWE_BOOT_MAGIC};
use aweos_kernel::entry::{kernel_entry, KernelBootStatus};

const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 0x36D7_6289;
const MAX_MEMORY_REGIONS: usize = 128;

static mut MEMORY_REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] =
    [MemoryRegion { base: 0, length: 0, kind: 2, reserved: 0 }; MAX_MEMORY_REGIONS];

#[used]
#[unsafe(link_section = ".multiboot2_header")]
#[unsafe(no_mangle)]
static MULTIBOOT2_HEADER: [u32; 4] = [0xE85250D6, 0, 16, 0x17ADAF1A];

// The bare-metal boot entry must not be linked into Cargo's host-side test
// harness, which supplies its own `_start`. The boot-image build is non-test
// and therefore keeps the complete Multiboot2 entry point.
#[cfg(not(test))]
global_asm!(r#"
.section .text.boot
.global _start
.type _start, @function
_start:
    cli
    lea rsp, [rip + stack_top]
    and rsp, -16
    xor rbp, rbp
    mov edi, eax
    mov esi, ebx
    call rust_main
.Lhalt:
    hlt
    jmp .Lhalt
.size _start, .-_start
.section .bss.stack,"aw",@nobits
.align 16
stack_bottom:
.skip 65536
stack_top:
"#);

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(boot_magic: u32, boot_info_addr: u32) -> ! {
    serial_init();
    serial_write(b"AWEOS CellKernel\r\n");
    serial_write(b"AWEOS boot: x86_64 Multiboot2 entry\r\n");

    let info = if boot_magic == MULTIBOOT2_BOOTLOADER_MAGIC {
        parse_multiboot2(boot_info_addr as usize)
    } else {
        serial_write(b"AWEOS: invalid Multiboot2 boot magic\r\n");
        BootInfo::empty(Architecture::X86_64)
    };

    match kernel_entry(&info) {
        KernelBootStatus::Ready => {
            serial_write(b"AWEOS: boot protocol validated\r\n");
            serial_write(b"AWEOS: kernel state = RUNNING\r\n");
        }
        KernelBootStatus::InvalidBootInfo => serial_write(b"AWEOS: invalid boot info\r\n"),
        KernelBootStatus::UnsupportedArchitecture => {
            serial_write(b"AWEOS: unsupported architecture\r\n")
        }
        KernelBootStatus::NoCpu => serial_write(b"AWEOS: no CPU reported\r\n"),
        KernelBootStatus::NoUsableMemory => {
            serial_write(b"AWEOS: no usable memory reported\r\n")
        }
    }

    serial_write(b"AWEOS: kernel is alive\r\n");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

#[cfg(not(test))]
fn parse_multiboot2(addr: usize) -> BootInfo {
    if addr == 0 {
        return BootInfo::empty(Architecture::X86_64);
    }

    let total_size = unsafe { core::ptr::read_unaligned(addr as *const u32) } as usize;
    if !(16..=1024 * 1024).contains(&total_size) {
        return BootInfo::empty(Architecture::X86_64);
    }

    let regions_ptr = core::ptr::addr_of_mut!(MEMORY_REGIONS) as *mut MemoryRegion;
    let mut region_count = 0u32;
    let (
        mut framebuffer_address,
        mut framebuffer_size,
        mut framebuffer_width,
        mut framebuffer_height,
        mut framebuffer_pitch,
        mut acpi_rsdp,
    ) = (0u64, 0u64, 0u32, 0u32, 0u32, 0u64);

    let mut offset = 8usize;
    while offset + 8 <= total_size {
        let tag = unsafe { core::ptr::read_unaligned((addr + offset) as *const u32) };
        let size = unsafe { core::ptr::read_unaligned((addr + offset + 4) as *const u32) } as usize;
        if size < 8 || offset + size > total_size || tag == 0 {
            break;
        }

        match tag {
            6 if size >= 16 => {
                let entry_size = unsafe {
                    core::ptr::read_unaligned((addr + offset + 8) as *const u32)
                } as usize;
                if entry_size >= 24 {
                    let mut entry = offset + 16;
                    while entry + entry_size <= offset + size
                        && (region_count as usize) < MAX_MEMORY_REGIONS
                    {
                        let base = unsafe {
                            core::ptr::read_unaligned((addr + entry) as *const u64)
                        };
                        let length = unsafe {
                            core::ptr::read_unaligned((addr + entry + 8) as *const u64)
                        };
                        let kind = unsafe {
                            core::ptr::read_unaligned((addr + entry + 16) as *const u32)
                        };
                        unsafe {
                            core::ptr::write(
                                regions_ptr.add(region_count as usize),
                                MemoryRegion { base, length, kind, reserved: 0 },
                            );
                        }
                        region_count += 1;
                        entry += entry_size;
                    }
                }
            }
            8 if size >= 32 => {
                framebuffer_address = unsafe {
                    core::ptr::read_unaligned((addr + offset + 8) as *const u64)
                };
                framebuffer_pitch = unsafe {
                    core::ptr::read_unaligned((addr + offset + 16) as *const u32)
                };
                framebuffer_width = unsafe {
                    core::ptr::read_unaligned((addr + offset + 20) as *const u32)
                };
                framebuffer_height = unsafe {
                    core::ptr::read_unaligned((addr + offset + 24) as *const u32)
                };
                framebuffer_size = (framebuffer_pitch as u64)
                    .saturating_mul(framebuffer_height as u64);
            }
            14 | 15 if size >= 16 => {
                acpi_rsdp = (addr + offset + 8) as u64;
            }
            _ => {}
        }

        offset = (offset + size + 7) & !7;
    }

    BootInfo {
        magic: AWE_BOOT_MAGIC,
        version: awe_boot_protocol::AWE_BOOT_VERSION,
        size: core::mem::size_of::<BootInfo>() as u32,
        architecture: Architecture::X86_64,
        cpu_count: 1,
        memory_regions: regions_ptr,
        memory_region_count: region_count,
        framebuffer_address,
        framebuffer_size,
        framebuffer_width,
        framebuffer_height,
        framebuffer_pitch,
        acpi_rsdp,
        device_tree: 0,
        kernel_base: 0,
        kernel_size: 0,
    }
}

#[cfg(not(test))]
fn serial_init() {
    unsafe {
        core::arch::asm!(
            "mov dx, 0x3F9",
            "xor al, al",
            "out dx, al",
            "mov dx, 0x3FB",
            "mov al, 0x80",
            "out dx, al",
            "mov dx, 0x3F8",
            "mov al, 3",
            "out dx, al",
            "mov dx, 0x3FB",
            "mov al, 3",
            "out dx, al",
            "mov dx, 0x3FA",
            "mov al, 0xC7",
            "out dx, al",
            "mov dx, 0x3FC",
            "mov al, 0x0B",
            "out dx, al",
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

#[cfg(not(test))]
fn serial_write(bytes: &[u8]) {
    for &byte in bytes {
        unsafe {
            core::arch::asm!(
                "mov dx, 0x3FD",
                "2: in al, dx",
                "test al, 0x20",
                "jz 2b",
                "mov dx, 0x3F8",
                "mov al, cl",
                "out dx, al",
                in("cl") byte,
                out("al") _,
                out("dx") _,
                options(nostack, preserves_flags)
            );
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    serial_write(b"AWEOS: KERNEL PANIC\r\n");
    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack, preserves_flags)); }
    }
}
