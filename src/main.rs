#![no_std]
#![no_main]

mod arch;
mod awosa;
mod ipc;
mod memory;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Early kernel entry. Boot firmware/loader owns platform discovery;
    // later stages initialize memory, interrupts, scheduler and services.
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
