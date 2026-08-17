#![no_std]
#![cfg_attr(not(test), no_main)]

use awe_boot_protocol::MemoryRegion;

const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 0x36D7_6289;
const MAX_MEMORY_REGIONS: usize = 128;
const FALLBACK_MEMORY_BASE: u64 = 0x0100_0000;
const FALLBACK_MEMORY_LENGTH: u64 = 0x0200_0000;
const MULTIBOOT2_MIN_SIZE: usize = 16;
const MULTIBOOT2_MAX_SIZE: usize = 1024 * 1024;

static mut MEMORY_REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] = [MemoryRegion {
    base: 0,
    length: 0,
    kind: 2,
    reserved: 0,
}; MAX_MEMORY_REGIONS];

#[used]
#[unsafe(link_section = ".multiboot2_header")]
#[unsafe(no_mangle)]
static MULTIBOOT2_HEADER: [u32; 4] = [0xE852_50D6, 0, 16, 0x17AD_AF1A];

#[cfg(not(test))]
#[unsafe(link_section = ".bss.boot")]
#[unsafe(no_mangle)]
static mut BOOT_PML4: [u64; 512] = [0; 512];

#[cfg(not(test))]
#[unsafe(link_section = ".bss.boot")]
#[unsafe(no_mangle)]
static mut BOOT_PDPT: [u64; 512] = [0; 512];

#[cfg(not(test))]
#[unsafe(link_section = ".bss.boot")]
#[unsafe(no_mangle)]
static mut BOOT_PD: [u64; 512] = [0; 512];

#[cfg(not(test))]
#[unsafe(link_section = ".bss.stack")]
#[unsafe(no_mangle)]
static mut BOOT_STACK: [u8; 65536] = [0; 65536];

#[cfg(not(test))]
core::arch::global_asm!(
    r#"
.intel_syntax noprefix
.code32
.section .text.boot
.global _start
.type _start, @function
_start:
    cli

    # Multiboot2 enters the kernel in 32-bit protected mode with
    # EAX = boot magic and EBX = Multiboot2 information pointer.
    mov dword ptr [boot_magic_saved], eax
    mov dword ptr [boot_info_saved], ebx

    # Build a minimal identity map for the first 1 GiB using 2 MiB pages.
    # PML4[0] -> PDPT[0] -> PD[0..511].
    mov eax, offset BOOT_PDPT
    or eax, 3
    mov dword ptr [BOOT_PML4], eax
    mov dword ptr [BOOT_PML4 + 4], 0

    mov eax, offset BOOT_PD
    or eax, 3
    mov dword ptr [BOOT_PDPT], eax
    mov dword ptr [BOOT_PDPT + 4], 0

    xor ecx, ecx
1:
    mov eax, ecx
    shl eax, 21
    or eax, 0x83
    mov dword ptr [BOOT_PD + ecx * 8], eax
    mov dword ptr [BOOT_PD + ecx * 8 + 4], 0
    inc ecx
    cmp ecx, 512
    jb 1b

    mov eax, offset BOOT_PML4
    mov cr3, eax

    # Enable PAE and long mode.
    mov eax, cr4
    or eax, 0x20
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100
    wrmsr

    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax

    # Temporary 64-bit GDT: null, 64-bit code, data.
    lgdt [gdt64_descriptor]

    # Encode a 32-bit protected-mode far jump directly. The destination
    # is below 4 GiB because the linker places the image at 1 MiB.
    .byte 0xEA
    .long long_mode_entry
    .word 0x08

.code64
long_mode_entry:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    lea rsp, [rip + BOOT_STACK + 65536]
    and rsp, -16
    xor rbp, rbp

    mov edi, dword ptr [rip + boot_magic_saved]
    mov esi, dword ptr [rip + boot_info_saved]
    call rust_main

.Lhalt:
    hlt
    jmp .Lhalt

.align 8
boot_magic_saved:
    .long 0
boot_info_saved:
    .long 0

.align 8
gdt64:
    .quad 0
    .quad 0x00AF9A000000FFFF
gdt64_data:
    .quad 0x00CF92000000FFFF
gdt64_end:
gdt64_descriptor:
    .word gdt64_end - gdt64 - 1
    .quad gdt64

.size _start, .-_start
.att_syntax prefix
"#
);

#[cfg(test)]
fn main() {}

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(boot_magic: u32, boot_info_addr: u64) -> ! {
    serial_init();
    serial_write(b"AWEOS CellKernel\r\n");
    serial_write(b"AWEOS boot: x86_64 Multiboot2 entry\r\n");

    if boot_magic != MULTIBOOT2_BOOTLOADER_MAGIC {
        serial_write(b"AWEOS: Multiboot2 magic mismatch\r\n");
        serial_write(b"AWEOS: boot handoff rejected\r\n");
        halt_forever();
    }

    if boot_info_addr == 0 {
        serial_write(b"AWEOS: invalid Multiboot2 handoff\r\n");
        halt_forever();
    }

    let info = parse_multiboot2(boot_info_addr as usize);

    match kernel_entry(&info) {
        KernelBootStatus::Ready => {
            serial_write(b"AWEOS: boot protocol validated\r\n");
            #[cfg(target_arch = "x86_64")]
            unsafe {
                activate_bootstrap_identity_map();
            }
            serial_write(b"AWEOS: x86_64 bootstrap paging = ACTIVE\r\n");
            serial_write(b"AWEOS: kernel state = RUNNING\r\n");
        }
        KernelBootStatus::InvalidBootInfo => {
            serial_write(b"AWEOS: invalid boot info\r\n");
            halt_forever();
        }
        KernelBootStatus::UnsupportedArchitecture => {
            serial_write(b"AWEOS: unsupported architecture\r\n");
            halt_forever();
        }
        KernelBootStatus::NoCpu => {
            serial_write(b"AWEOS: no CPU reported\r\n");
            halt_forever();
        }
        KernelBootStatus::NoUsableMemory => {
            serial_write(b"AWEOS: no usable memory reported\r\n");
            halt_forever();
        }
    }

    serial_write(b"AWEOS: kernel is alive\r\n");
    halt_forever();
}

#[cfg(not(test))]
fn halt_forever() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(not(test))]
fn parse_multiboot2(addr: usize) -> BootInfo {
    if addr == 0 {
        return BootInfo::empty(Architecture::X86_64);
    }

    let total_size = unsafe { core::ptr::read_unaligned(addr as *const u32) } as usize;
    if !(MULTIBOOT2_MIN_SIZE..=MULTIBOOT2_MAX_SIZE).contains(&total_size) {
        return BootInfo::empty(Architecture::X86_64);
    }

    let regions_ptr = core::ptr::addr_of_mut!(MEMORY_REGIONS) as *mut MemoryRegion;
    let mut region_count = 0u32;
    let mut basic_mem_upper_kb = 0u64;
    let (
        mut framebuffer_address,
        mut framebuffer_size,
        mut framebuffer_width,
        mut framebuffer_height,
        mut framebuffer_pitch,
        mut acpi_rsdp,
    ) = (0u64, 0u64, 0u32, 0u32, 0u32, 0u64);
    let mut saw_end_tag = false;

    let mut offset = 8usize;
    while offset + 8 <= total_size {
        let tag = unsafe { core::ptr::read_unaligned((addr + offset) as *const u32) };
        let size = unsafe { core::ptr::read_unaligned((addr + offset + 4) as *const u32) } as usize;

        if tag == 0 {
            if size == 8 {
                saw_end_tag = true;
            }
            break;
        }
        if size < 8 || offset.saturating_add(size) > total_size {
            return BootInfo::empty(Architecture::X86_64);
        }

        match tag {
            4 if size >= 16 => {
                basic_mem_upper_kb =
                    unsafe { core::ptr::read_unaligned((addr + offset + 12) as *const u32) } as u64;
            }
            6 if size >= 16 => {
                let entry_size =
                    unsafe { core::ptr::read_unaligned((addr + offset + 8) as *const u32) }
                        as usize;
                if entry_size >= 24 {
                    let mut entry = offset + 16;
                    while entry_size <= size.saturating_sub(16)
                        && entry + entry_size <= offset + size
                        && (region_count as usize) < MAX_MEMORY_REGIONS
                    {
                        let base =
                            unsafe { core::ptr::read_unaligned((addr + entry) as *const u64) };
                        let length =
                            unsafe { core::ptr::read_unaligned((addr + entry + 8) as *const u64) };
                        let kind =
                            unsafe { core::ptr::read_unaligned((addr + entry + 16) as *const u32) };
                        unsafe {
                            core::ptr::write(
                                regions_ptr.add(region_count as usize),
                                MemoryRegion {
                                    base,
                                    length,
                                    kind,
                                    reserved: 0,
                                },
                            );
                        }
                        region_count += 1;
                        entry += entry_size;
                    }
                }
            }
            8 if size >= 32 => {
                framebuffer_address =
                    unsafe { core::ptr::read_unaligned((addr + offset + 8) as *const u64) };
                framebuffer_pitch =
                    unsafe { core::ptr::read_unaligned((addr + offset + 16) as *const u32) };
                framebuffer_width =
                    unsafe { core::ptr::read_unaligned((addr + offset + 20) as *const u32) };
                framebuffer_height =
                    unsafe { core::ptr::read_unaligned((addr + offset + 24) as *const u32) };
                framebuffer_size =
                    (framebuffer_pitch as u64).saturating_mul(framebuffer_height as u64);
            }
            14 | 15 if size >= 16 => {
                acpi_rsdp = (addr + offset + 8) as u64;
            }
            _ => {}
        }

        offset = offset.saturating_add(size + 7) & !7;
    }

    if !saw_end_tag {
        return BootInfo::empty(Architecture::X86_64);
    }

    let mut has_usable = false;
    for index in 0..region_count as usize {
        let region = unsafe { core::ptr::read(regions_ptr.add(index)) };
        if region.kind == 1 && region.length >= 4096 {
            has_usable = true;
            break;
        }
    }

    if !has_usable {
        let length = basic_mem_upper_kb.saturating_mul(1024);
        unsafe {
            core::ptr::write(
                regions_ptr,
                MemoryRegion {
                    base: if length >= 4096 {
                        0x0010_0000
                    } else {
                        FALLBACK_MEMORY_BASE
                    },
                    length: if length >= 4096 {
                        length
                    } else {
                        FALLBACK_MEMORY_LENGTH
                    },
                    kind: 1,
                    reserved: 0,
                },
            );
        }
        region_count = 1;
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

#[cfg(not(test))]
fn serial_write(bytes: &[u8]) {
    for &byte in bytes {
        unsafe {
            core::arch::asm!(
                "mov dx, 0x3FD", "2: in al, dx", "test al, 0x20", "jz 2b",
                "mov dx, 0x3F8", "mov al, cl", "out dx, al",
                in("cl") byte, out("al") _, out("dx") _, options(nostack, preserves_flags)
            );
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    serial_write(b"AWEOS: KERNEL PANIC\r\n");
    halt_forever();
}
