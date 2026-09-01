#![no_std]

use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, AtomicU64, Ordering};
use crate::drivers::{KeyCode, Ps2Event};

pub const CAPACITY: usize = 128;
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);
static QUEUE: [AtomicU64; CAPACITY] = [const { AtomicU64::new(0) }; CAPACITY];
static KB_EXTENDED: AtomicBool = AtomicBool::new(false);
static KB_BREAK: AtomicBool = AtomicBool::new(false);
static MOUSE_INDEX: AtomicU8 = AtomicU8::new(0);
static MOUSE_0: AtomicU8 = AtomicU8::new(0);
static MOUSE_1: AtomicU8 = AtomicU8::new(0);
static MOUSE_2: AtomicU8 = AtomicU8::new(0);

fn enqueue(encoded: u64) -> bool {
    let tail = TAIL.load(Ordering::Relaxed);
    let next = (tail + 1) % CAPACITY;
    if next == HEAD.load(Ordering::Acquire) { return false; }
    QUEUE[tail].store(encoded, Ordering::Release);
    TAIL.store(next, Ordering::Release);
    true
}

pub fn dequeue() -> Option<Ps2Event> {
    let head = HEAD.load(Ordering::Relaxed);
    if head == TAIL.load(Ordering::Acquire) { return None; }
    let encoded = QUEUE[head].load(Ordering::Acquire);
    HEAD.store((head + 1) % CAPACITY, Ordering::Release);
    if encoded >> 63 == 0 {
        let code = (encoded & 0xffff) as u8;
        let pressed = ((encoded >> 16) & 1) != 0;
        Some(Ps2Event::Key { code: map_key(code), pressed })
    } else {
        let dx = (encoded as u16) as i16;
        let dy = ((encoded >> 16) as u16) as i16;
        let buttons = ((encoded >> 32) & 0xff) as u8;
        Some(Ps2Event::Pointer { dx, dy, buttons })
    }
}

fn map_key(code: u8) -> KeyCode {
    match code {
        0x01 => KeyCode::Escape, 0x0D => KeyCode::Tab, 0x1C => KeyCode::Enter,
        0x0E => KeyCode::Backspace, 0x39 => KeyCode::Space, 0x4B => KeyCode::Left,
        0x4D => KeyCode::Right, 0x48 => KeyCode::Up, 0x50 => KeyCode::Down,
        other => KeyCode::Unknown(other),
    }
}

pub fn irq_keyboard_byte(byte: u8) -> bool {
    match byte {
        0xE0 => { KB_EXTENDED.store(true, Ordering::Relaxed); false }
        0xF0 => { KB_BREAK.store(true, Ordering::Relaxed); false }
        code => {
            let extended = KB_EXTENDED.swap(false, Ordering::Relaxed);
            let pressed = !KB_BREAK.swap(false, Ordering::Relaxed);
            let code = if extended { code } else { code };
            let mut encoded = code as u64;
            if pressed { encoded |= 1 << 16; }
            enqueue(encoded)
        }
    }
}

pub fn irq_mouse_byte(byte: u8) -> bool {
    let index = MOUSE_INDEX.load(Ordering::Relaxed) as usize;
    if index == 0 && byte & 0x08 == 0 { return false; }
    match index {
        0 => MOUSE_0.store(byte, Ordering::Relaxed),
        1 => MOUSE_1.store(byte, Ordering::Relaxed),
        2 => MOUSE_2.store(byte, Ordering::Relaxed),
        _ => return false,
    }
    if index < 2 { MOUSE_INDEX.store((index + 1) as u8, Ordering::Relaxed); return false; }
    MOUSE_INDEX.store(0, Ordering::Relaxed);
    let flags = MOUSE_0.load(Ordering::Relaxed);
    if flags & 0xC0 != 0 { return false; }
    let raw_x = MOUSE_1.load(Ordering::Relaxed) as i16;
    let raw_y = MOUSE_2.load(Ordering::Relaxed) as i16;
    let dx = if flags & 0x10 != 0 { raw_x - 256 } else { raw_x };
    let dy_raw = if flags & 0x20 != 0 { raw_y - 256 } else { raw_y };
    let dy = -dy_raw;
    let encoded = (dx as u16 as u64) | ((dy as u16 as u64) << 16) | (((flags & 7) as u64) << 32) | (1u64 << 63);
    enqueue(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyboard_irq_generates_decoded_event() {
        assert!(irq_keyboard_byte(0x1C));
        assert_eq!(dequeue(), Some(Ps2Event::Key { code: KeyCode::Enter, pressed: true }));
        irq_keyboard_byte(0xF0); irq_keyboard_byte(0x1C);
        assert_eq!(dequeue(), Some(Ps2Event::Key { code: KeyCode::Enter, pressed: false }));
    }
    #[test]
    fn mouse_irq_generates_signed_event() {
        assert!(!irq_mouse_byte(0x09)); assert!(!irq_mouse_byte(0xFE)); assert!(irq_mouse_byte(0x02));
        assert_eq!(dequeue(), Some(Ps2Event::Pointer { dx: -2, dy: -2, buttons: 1 }));
    }
}