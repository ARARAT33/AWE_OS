#![no_std]

use super::idt::Idt;

pub const TIMER_VECTOR: u8 = 32;

/// Installs the minimal early interrupt surface. The timer handler remains an
/// explicit ABI boundary so scheduler code can be attached without changing
/// IDT construction.
pub fn install_early_interrupts(idt: &mut Idt, code_selector: u16, timer_handler: u64) {
    idt.set_handler(TIMER_VECTOR, timer_handler, code_selector);
}

#[cfg(test)]
mod tests {
    use super::*;
    extern "C" fn timer() {}

    #[test]
    fn timer_vector_is_installed() {
        let mut idt = Idt::new();
        install_early_interrupts(&mut idt, 0x08, timer as *const () as usize as u64);
        assert!(idt.is_present(TIMER_VECTOR));
        assert!(!idt.is_present(33));
    }
}
