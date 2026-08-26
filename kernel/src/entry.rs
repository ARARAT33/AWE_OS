#![no_std]

use awe_boot_protocol::{validate, BootInfo};
use crate::boot_phase::{BootPhase, BootProgress};
use crate::memory::PhysicalFrameAllocator;
use crate::runtime::{CapabilitySet, EndUserRuntime, FramebufferInfo, RuntimeContext};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KernelBootStatus { Ready = 0, InvalidBootInfo = 1, UnsupportedArchitecture = 2, NoCpu = 3, NoUsableMemory = 4, InvalidFramebuffer = 5 }

pub struct KernelContext { progress: BootProgress }
impl Default for KernelContext { fn default() -> Self { Self::new() } }
impl KernelContext {
    pub const fn new() -> Self { Self { progress: BootProgress::new() } }
    pub const fn phase(&self) -> BootPhase { self.progress.phase() }
    pub fn advance(&mut self) -> bool { self.progress.advance() }
}

pub fn kernel_entry(info: &BootInfo) -> KernelBootStatus {
    if !validate(info) { return KernelBootStatus::InvalidBootInfo; }
    if !info.architecture.is_supported() { return KernelBootStatus::UnsupportedArchitecture; }
    if info.cpu_count == 0 { return KernelBootStatus::NoCpu; }
    if info.memory_region_count == 0 || info.memory_regions.is_null() { return KernelBootStatus::NoUsableMemory; }
    let mut frames = unsafe { PhysicalFrameAllocator::from_boot_info(info) };
    if frames.allocate().is_none() { return KernelBootStatus::NoUsableMemory; }

    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    {
        use crate::arch::x86_64::gdt::{init_gdt, KERNEL_CODE_SELECTOR, USER_CODE_SELECTOR};
        use crate::arch::x86_64::idt::IDT;
        use crate::arch::x86_64::interrupts::init_pic;
        use crate::arch::x86_64::isr_stubs::init_idt_stubs;
        use crate::arch::x86_64::serial_write_str;
        use crate::memory::allocator::init_kernel_heap;
        use crate::platform::pit::Pit;
        use crate::syscall::init_msr_syscall;

        static mut KERNEL_STACK: [u8; 65536] = [0; 65536];
        static mut USER_STACK: [u8; 16384] = [0; 16384];
        let kernel_stack_top = core::ptr::addr_of_mut!(KERNEL_STACK) as u64 + 65536;
        let user_stack_top = core::ptr::addr_of_mut!(USER_STACK) as u64 + 16384;

        init_gdt(kernel_stack_top);
        serial_write_str("AWEOS: GDT/TSS initialized\r\n");
        unsafe { init_msr_syscall(userspace_entry as *const () as usize as u64, KERNEL_CODE_SELECTOR, USER_CODE_SELECTOR); }
        let idt_ptr = core::ptr::addr_of_mut!(IDT);
        unsafe { init_idt_stubs(&mut *idt_ptr, KERNEL_CODE_SELECTOR); (*idt_ptr).load(); }
        init_kernel_heap();
        unsafe { init_pic(); }
        if let Some(pit) = Pit::new(1000) { unsafe { pit.program(); } }
        serial_write_str("AWEOS: core interrupt/timer path initialized\r\n");

        let mut runtime = EndUserRuntime::new();
        if info.framebuffer_address != 0 && info.framebuffer_size != 0 {
            let fb = FramebufferInfo {
                address: info.framebuffer_address,
                size: info.framebuffer_size,
                width: info.framebuffer_width,
                height: info.framebuffer_height,
                pitch: info.framebuffer_pitch,
                bytes_per_pixel: 4,
            };
            if runtime.attach_framebuffer(fb).is_err() { return KernelBootStatus::InvalidFramebuffer; }
            serial_write_str("AWEOS: BootInfo framebuffer validated\r\n");
        }

        let _ = runtime.declare_service(1, CapabilitySet::DEVICE.union(CapabilitySet::IPC), 3);
        let _ = runtime.declare_service(2, CapabilitySet::STORAGE.union(CapabilitySet::IPC), 3);
        let _ = runtime.declare_service(3, CapabilitySet::NETWORK.union(CapabilitySet::IPC), 3);
        let _ = runtime.declare_service(4, CapabilitySet::IPC, 3);
        let _ = runtime.declare_service(5, CapabilitySet::PROCESS.union(CapabilitySet::IPC), 3);
        let _ = runtime.declare_service(6, CapabilitySet::UI.union(CapabilitySet::IPC), 3);
        serial_write_str("AWEOS: end-user runtime control plane prepared\r\n");
        unsafe { enter_userspace(userspace_entry as *const () as usize as u64, user_stack_top); }
    }
    KernelBootStatus::Ready
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
#[unsafe(no_mangle)]
pub extern "C" fn userspace_entry() -> ! {
    let message = b"AWEOS: Ring 3 active; device access is mediated by syscall/IPC.\r\n";
    let mut process = crate::process::ProcessDescriptor {
        id: crate::process::ProcessId(1), state: crate::process::ProcessState::Running,
        budget: crate::process::ResourceBudget { cpu_ticks: 1000, memory_bytes: 1024 * 1024, ipc_messages: 64 },
    };
    let mut syscall = crate::syscall::SyscallContext { process: &mut process };
    let _ = syscall.dispatch(8, [message.as_ptr() as u64, message.len() as u64, 0, 0, 0, 0]);
    loop { let _ = syscall.dispatch(0, [0; 6]); unsafe { core::arch::asm!("pause"); } }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub unsafe fn enter_userspace(user_rip: u64, user_rsp: u64) {
    use crate::arch::x86_64::gdt::{USER_CODE_SELECTOR, USER_DATA_SELECTOR};
    let user_cs = USER_CODE_SELECTOR as u64; let user_ss = USER_DATA_SELECTOR as u64;
    // IF=1, IOPL=0: Ring 3 cannot execute privileged device I/O.
    let rflags = 0x0202u64;
    unsafe { core::arch::asm!("push {0}", "push {1}", "push {2}", "push {3}", "push {4}", "iretq", in(reg) user_ss, in(reg) user_rsp, in(reg) rflags, in(reg) user_cs, in(reg) user_rip, options(noreturn)); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use awe_boot_protocol::{Architecture, BootInfo, MemoryRegion};
    fn info() -> (BootInfo, [MemoryRegion; 1]) {
        let regions = [MemoryRegion { base: 0x1000, length: 0x10000, kind: 1, reserved: 0 }];
        let boot = BootInfo { magic: awe_boot_protocol::AWE_BOOT_MAGIC, version: awe_boot_protocol::AWE_BOOT_VERSION, size: core::mem::size_of::<BootInfo>() as u32, architecture: Architecture::X86_64, cpu_count: 1, memory_regions: regions.as_ptr(), memory_region_count: 1, framebuffer_address: 0, framebuffer_size: 0, framebuffer_width: 0, framebuffer_height: 0, framebuffer_pitch: 0, acpi_rsdp: 0, device_tree: 0, kernel_base: 0, kernel_size: 0 };
        (boot, regions)
    }
    #[test] fn invalid_boot_info_is_rejected() { let (mut boot, _) = info(); boot.magic = 0; assert_eq!(kernel_entry(&boot), KernelBootStatus::InvalidBootInfo); }
    #[test] fn valid_minimum_boot_info_is_accepted() { let (boot, _) = info(); assert_eq!(kernel_entry(&boot), KernelBootStatus::Ready); }
}
