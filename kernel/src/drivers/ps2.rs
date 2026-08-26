#![no_std]

use crate::arch::x86_64::{io_in8, io_out8};

pub const DATA_PORT: u16 = 0x60;
pub const STATUS_PORT: u16 = 0x64;
pub const COMMAND_PORT: u16 = 0x64;
pub const MAX_EVENTS: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ps2Error {
    Timeout,
    Controller,
    QueueFull,
    InvalidPacket,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyCode {
    Escape,
    Enter,
    Backspace,
    Tab,
    Space,
    Left,
    Right,
    Up,
    Down,
    Character(u8),
    Unknown(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ps2Event {
    Key { code: KeyCode, pressed: bool },
    Pointer { dx: i16, dy: i16, buttons: u8 },
}

pub struct EventQueue {
    slots: [Option<Ps2Event>; MAX_EVENTS],
    head: usize,
    len: usize,
}

impl EventQueue {
    pub const fn new() -> Self {
        Self {
            slots: [None; MAX_EVENTS],
            head: 0,
            len: 0,
        }
    }

    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }

    pub fn push(&mut self, event: Ps2Event) -> Result<(), Ps2Error> {
        if self.len == MAX_EVENTS { return Err(Ps2Error::QueueFull); }
        let index = (self.head + self.len) % MAX_EVENTS;
        self.slots[index] = Some(event);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Ps2Event> {
        if self.len == 0 { return None; }
        let event = self.slots[self.head].take();
        self.head = (self.head + 1) % MAX_EVENTS;
        self.len -= 1;
        event
    }
}

impl Default for EventQueue { fn default() -> Self { Self::new() } }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyboardDecoder {
    extended: bool,
    break_code: bool,
}

impl KeyboardDecoder {
    pub const fn new() -> Self { Self { extended: false, break_code: false } }

    pub fn feed(&mut self, byte: u8) -> Option<Ps2Event> {
        match byte {
            0xE0 => { self.extended = true; None }
            0xF0 => { self.break_code = true; None }
            code => {
                let pressed = !self.break_code;
                let mapped = if self.extended {
                    match code {
                        0x4B => KeyCode::Left,
                        0x4D => KeyCode::Right,
                        0x48 => KeyCode::Up,
                        0x50 => KeyCode::Down,
                        other => KeyCode::Unknown(other),
                    }
                } else {
                    match code {
                        0x01 => KeyCode::Escape,
                        0x0D => KeyCode::Tab,
                        0x1C => KeyCode::Enter,
                        0x0E => KeyCode::Backspace,
                        0x39 => KeyCode::Space,
                        0x10..=0x35 => KeyCode::Character(code),
                        other => KeyCode::Unknown(other),
                    }
                };
                self.extended = false;
                self.break_code = false;
                Some(Ps2Event::Key { code: mapped, pressed })
            }
        }
    }
}

impl Default for KeyboardDecoder { fn default() -> Self { Self::new() } }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MouseDecoder {
    packet: [u8; 3],
    index: usize,
}

impl MouseDecoder {
    pub const fn new() -> Self { Self { packet: [0; 3], index: 0 } }

    pub fn feed(&mut self, byte: u8) -> Result<Option<Ps2Event>, Ps2Error> {
        if self.index == 0 && byte & 0x08 == 0 {
            return Ok(None);
        }
        self.packet[self.index] = byte;
        self.index += 1;
        if self.index < 3 { return Ok(None); }
        self.index = 0;
        let flags = self.packet[0];
        let x = self.packet[1] as i16;
        let y = self.packet[2] as i16;
        let dx = if flags & 0x10 != 0 { x - 256 } else { x };
        let dy_raw = if flags & 0x20 != 0 { y - 256 } else { y };
        let dy = -dy_raw;
        if flags & 0xC0 != 0 { return Err(Ps2Error::InvalidPacket); }
        Ok(Some(Ps2Event::Pointer { dx, dy, buttons: flags & 0x07 }))
    }
}

impl Default for MouseDecoder { fn default() -> Self { Self::new() } }

pub struct Controller {
    pub events: EventQueue,
    pub keyboard: KeyboardDecoder,
    pub mouse: MouseDecoder,
    pub initialized: bool,
}

impl Controller {
    pub const fn new() -> Self {
        Self {
            events: EventQueue::new(),
            keyboard: KeyboardDecoder::new(),
            mouse: MouseDecoder::new(),
            initialized: false,
        }
    }

    /// Initialize the legacy 8042 controller with bounded polling time.
    pub unsafe fn init(&mut self) -> Result<(), Ps2Error> {
        const LIMIT: usize = 100_000;
        let mut spins = 0;
        while unsafe { io_in8(STATUS_PORT) } & 0x01 != 0 {
            let _ = unsafe { io_in8(DATA_PORT) };
            spins += 1;
            if spins == LIMIT { return Err(Ps2Error::Timeout); }
        }
        unsafe { io_out8(COMMAND_PORT, 0xAE); } // enable first PS/2 port
        self.initialized = true;
        Ok(())
    }

    /// Poll all currently available bytes. This function never blocks forever.
    pub unsafe fn poll(&mut self, max_bytes: usize) -> Result<usize, Ps2Error> {
        if !self.initialized { return Err(Ps2Error::Controller); }
        let mut read = 0;
        while read < max_bytes {
            let status = unsafe { io_in8(STATUS_PORT) };
            if status & 0x01 == 0 { break; }
            let byte = unsafe { io_in8(DATA_PORT) };
            // Bit 5 distinguishes the auxiliary mouse port on the legacy controller.
            if status & 0x20 != 0 {
                if let Some(event) = self.mouse.feed(byte)? { self.events.push(event)?; }
            } else if let Some(event) = self.keyboard.feed(byte) {
                self.events.push(event)?;
            }
            read += 1;
        }
        Ok(read)
    }
}

impl Default for Controller { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_decodes_press_release_and_extended_keys() {
        let mut d = KeyboardDecoder::new();
        assert_eq!(d.feed(0x1C), Some(Ps2Event::Key { code: KeyCode::Enter, pressed: true }));
        assert_eq!(d.feed(0xF0), None);
        assert_eq!(d.feed(0x1C), Some(Ps2Event::Key { code: KeyCode::Enter, pressed: false }));
        assert_eq!(d.feed(0xE0), None);
        assert_eq!(d.feed(0x4B), Some(Ps2Event::Key { code: KeyCode::Left, pressed: true }));
    }

    #[test]
    fn mouse_decodes_signed_deltas_and_buttons() {
        let mut d = MouseDecoder::new();
        assert_eq!(d.feed(0x09).unwrap(), None);
        assert_eq!(d.feed(0xFE).unwrap(), None);
        assert_eq!(d.feed(0x02).unwrap(), Some(Ps2Event::Pointer { dx: -2, dy: -2, buttons: 1 }));
    }

    #[test]
    fn malformed_mouse_packet_is_rejected() {
        let mut d = MouseDecoder::new();
        assert_eq!(d.feed(0xC8).unwrap_err(), Ps2Error::InvalidPacket);
    }

    #[test]
    fn event_queue_is_bounded_fifo() {
        let mut q = EventQueue::new();
        q.push(Ps2Event::Key { code: KeyCode::Escape, pressed: true }).unwrap();
        assert_eq!(q.pop(), Some(Ps2Event::Key { code: KeyCode::Escape, pressed: true }));
        assert!(q.pop().is_none());
    }
}