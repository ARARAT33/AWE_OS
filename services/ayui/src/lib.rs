#![no_std]

//! Bounded AYUI desktop primitives. Rendering backends remain outside the
//! compositor contract; this crate owns deterministic scene, window and input
//! validation that can be exercised without a GPU.

pub const MAX_WINDOWS: usize = 64;
pub const MAX_EVENTS: usize = 128;
pub const MAX_WIDTH: u32 = 16_384;
pub const MAX_HEIGHT: u32 = 16_384;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect { pub x: i32, pub y: i32, pub width: u32, pub height: u32 }

impl Rect {
    pub const fn valid(self) -> bool {
        self.width > 0 && self.height > 0 && self.width <= MAX_WIDTH && self.height <= MAX_HEIGHT
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window { pub id: WindowId, pub bounds: Rect, pub visible: bool, pub focused: bool }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent { Pointer { x: i32, y: i32, buttons: u8 }, Key { code: u16, pressed: bool } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiError { Full, InvalidRect, InvalidWindow, InvalidEvent }

pub struct Compositor {
    windows: [Option<Window>; MAX_WINDOWS],
    events: [Option<InputEvent>; MAX_EVENTS],
    event_head: usize,
    event_len: usize,
    next_id: u16,
}

impl Compositor {
    pub const fn new() -> Self {
        Self { windows: [None; MAX_WINDOWS], events: [None; MAX_EVENTS], event_head: 0, event_len: 0, next_id: 1 }
    }

    pub fn create_window(&mut self, bounds: Rect) -> Result<WindowId, UiError> {
        if !bounds.valid() { return Err(UiError::InvalidRect); }
        let slot = self.windows.iter().position(Option::is_none).ok_or(UiError::Full)?;
        let id = WindowId(self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or(UiError::Full)?;
        self.windows[slot] = Some(Window { id, bounds, visible: true, focused: false });
        Ok(id)
    }

    pub fn focus(&mut self, id: WindowId) -> Result<(), UiError> {
        let mut found = false;
        for window in self.windows.iter_mut().flatten() {
            if window.id == id { window.focused = true; found = true; } else { window.focused = false; }
        }
        if found { Ok(()) } else { Err(UiError::InvalidWindow) }
    }

    pub fn push_input(&mut self, event: InputEvent) -> Result<(), UiError> {
        if let InputEvent::Pointer { x, y, .. } = event {
            if x < -1_048_576 || y < -1_048_576 || x > 1_048_576 || y > 1_048_576 { return Err(UiError::InvalidEvent); }
        }
        if self.event_len == MAX_EVENTS { return Err(UiError::Full); }
        let index = (self.event_head + self.event_len) % MAX_EVENTS;
        self.events[index] = Some(event);
        self.event_len += 1;
        Ok(())
    }

    pub fn pop_input(&mut self) -> Option<InputEvent> {
        if self.event_len == 0 { return None; }
        let event = self.events[self.event_head].take();
        self.event_head = (self.event_head + 1) % MAX_EVENTS;
        self.event_len -= 1;
        event
    }

    pub const fn window_count(&self) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < MAX_WINDOWS { if self.windows[i].is_some() { count += 1; } i += 1; }
        count
    }
}

impl Default for Compositor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compositor_creates_and_focuses_windows() {
        let mut c = Compositor::new();
        let a = c.create_window(Rect { x: 0, y: 0, width: 800, height: 600 }).unwrap();
        let b = c.create_window(Rect { x: 10, y: 10, width: 400, height: 300 }).unwrap();
        c.focus(b).unwrap();
        assert_eq!(c.window_count(), 2);
        assert!(c.windows.iter().flatten().find(|w| w.id == b).unwrap().focused);
        assert!(!c.windows.iter().flatten().find(|w| w.id == a).unwrap().focused);
    }
    #[test]
    fn input_queue_is_bounded_fifo() {
        let mut c = Compositor::new();
        c.push_input(InputEvent::Key { code: 30, pressed: true }).unwrap();
        c.push_input(InputEvent::Key { code: 30, pressed: false }).unwrap();
        assert_eq!(c.pop_input(), Some(InputEvent::Key { code: 30, pressed: true }));
        assert_eq!(c.pop_input(), Some(InputEvent::Key { code: 30, pressed: false }));
        assert_eq!(c.pop_input(), None);
    }
    #[test]
    fn invalid_geometry_and_pointer_are_rejected() {
        let mut c = Compositor::new();
        assert_eq!(c.create_window(Rect { x: 0, y: 0, width: 0, height: 10 }), Err(UiError::InvalidRect));
        assert_eq!(c.push_input(InputEvent::Pointer { x: 2_000_000, y: 0, buttons: 0 }), Err(UiError::InvalidEvent));
    }
}
