//! Bounded AYUI desktop compositor, software rendering engine, widget system,
//! desktop shell, terminal emulator backend, and notification daemon.

#![no_std]

pub const MAX_WINDOWS: usize = 64;
pub const MAX_EVENTS: usize = 128;
pub const MAX_WIDTH: u32 = 16_384;
pub const MAX_HEIGHT: u32 = 16_384;
pub const MAX_NOTIFICATIONS: usize = 16;
pub const TERM_ROWS: usize = 25;
pub const TERM_COLS: usize = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn valid(self) -> bool {
        self.width > 0 && self.height > 0 && self.width <= MAX_WIDTH && self.height <= MAX_HEIGHT
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && px < self.x + (self.width as i32)
            && py >= self.y
            && py < self.y + (self.height as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 120,
        b: 215,
        a: 255,
    };
    pub const DARK_GRAY: Self = Self {
        r: 32,
        g: 32,
        b: 32,
        a: 255,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub id: WindowId,
    pub bounds: Rect,
    pub visible: bool,
    pub focused: bool,
    pub bg_color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    Pointer { x: i32, y: i32, buttons: u8 },
    Key { code: u16, pressed: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiError {
    Full,
    InvalidRect,
    InvalidWindow,
    InvalidEvent,
}

/// Software Framebuffer for display composition.
#[derive(Debug)]
pub struct Framebuffer<'a> {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub buffer: &'a mut [u8],
}

impl<'a> Framebuffer<'a> {
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        if rect.x < 0 || rect.y < 0 {
            return;
        }
        let start_x = rect.x as u32;
        let start_y = rect.y as u32;
        let end_x = (start_x + rect.width).min(self.width);
        let end_y = (start_y + rect.height).min(self.height);

        for y in start_y..end_y {
            for x in start_x..end_x {
                let offset = ((y * self.stride + x) * 4) as usize;
                if offset + 3 < self.buffer.len() {
                    self.buffer[offset] = color.b;
                    self.buffer[offset + 1] = color.g;
                    self.buffer[offset + 2] = color.r;
                    self.buffer[offset + 3] = color.a;
                }
            }
        }
    }
}

/// Widget system primitives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetType {
    Button { label_id: u16 },
    Label { text_len: u8 },
    Panel,
}

#[derive(Clone, Copy, Debug)]
pub struct Widget {
    pub id: u16,
    pub bounds: Rect,
    pub widget_type: WidgetType,
    pub active: bool,
}

/// Terminal Emulator Backend (80x25 Grid).
#[derive(Debug)]
pub struct TerminalBackend {
    pub grid: [[u8; TERM_COLS]; TERM_ROWS],
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl TerminalBackend {
    pub const fn new() -> Self {
        Self {
            grid: [[b' '; TERM_COLS]; TERM_ROWS],
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn write_char(&mut self, ch: u8) {
        match ch {
            b'\n' => {
                self.cursor_col = 0;
                self.cursor_row += 1;
            }
            b'\r' => {
                self.cursor_col = 0;
            }
            c => {
                if self.cursor_row < TERM_ROWS && self.cursor_col < TERM_COLS {
                    self.grid[self.cursor_row][self.cursor_col] = c;
                    self.cursor_col += 1;
                }
            }
        }
        if self.cursor_col >= TERM_COLS {
            self.cursor_col = 0;
            self.cursor_row += 1;
        }
        if self.cursor_row >= TERM_ROWS {
            self.scroll_up();
        }
    }

    fn scroll_up(&mut self) {
        for r in 1..TERM_ROWS {
            self.grid[r - 1] = self.grid[r];
        }
        self.grid[TERM_ROWS - 1] = [b' '; TERM_COLS];
        self.cursor_row = TERM_ROWS - 1;
    }
}

impl Default for TerminalBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification Item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Notification {
    pub id: u32,
    pub title_hash: u64,
    pub timestamp: u64,
    pub active: bool,
}

/// Notification Daemon Instance.
#[derive(Debug)]
pub struct NotificationDaemon {
    notifications: [Option<Notification>; MAX_NOTIFICATIONS],
    counter: u32,
}

impl NotificationDaemon {
    pub const fn new() -> Self {
        Self {
            notifications: [None; MAX_NOTIFICATIONS],
            counter: 1,
        }
    }

    pub fn push_notification(&mut self, title_hash: u64, timestamp: u64) -> Result<u32, UiError> {
        let nid = self.counter;
        for slot in self.notifications.iter_mut() {
            if slot.is_none() {
                *slot = Some(Notification {
                    id: nid,
                    title_hash,
                    timestamp,
                    active: true,
                });
                self.counter += 1;
                return Ok(nid);
            }
        }
        Err(UiError::Full)
    }
}

impl Default for NotificationDaemon {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Compositor {
    windows: [Option<Window>; MAX_WINDOWS],
    events: [Option<InputEvent>; MAX_EVENTS],
    event_head: usize,
    event_len: usize,
    next_id: u16,
}

impl Compositor {
    pub const fn new() -> Self {
        Self {
            windows: [None; MAX_WINDOWS],
            events: [None; MAX_EVENTS],
            event_head: 0,
            event_len: 0,
            next_id: 1,
        }
    }

    pub fn create_window(&mut self, bounds: Rect) -> Result<WindowId, UiError> {
        if !bounds.valid() {
            return Err(UiError::InvalidRect);
        }
        let slot = self
            .windows
            .iter()
            .position(Option::is_none)
            .ok_or(UiError::Full)?;
        let id = WindowId(self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or(UiError::Full)?;
        self.windows[slot] = Some(Window {
            id,
            bounds,
            visible: true,
            focused: false,
            bg_color: Color::DARK_GRAY,
        });
        Ok(id)
    }

    pub fn focus(&mut self, id: WindowId) -> Result<(), UiError> {
        let mut found = false;
        for window in self.windows.iter_mut().flatten() {
            if window.id == id {
                window.focused = true;
                found = true;
            } else {
                window.focused = false;
            }
        }
        if found {
            Ok(())
        } else {
            Err(UiError::InvalidWindow)
        }
    }

    pub fn push_input(&mut self, event: InputEvent) -> Result<(), UiError> {
        if let InputEvent::Pointer { x, y, .. } = event
            && (x < -1_048_576 || y < -1_048_576 || x > 1_048_576 || y > 1_048_576)
        {
            return Err(UiError::InvalidEvent);
        }
        if self.event_len == MAX_EVENTS {
            return Err(UiError::Full);
        }
        let index = (self.event_head + self.event_len) % MAX_EVENTS;
        self.events[index] = Some(event);
        self.event_len += 1;
        Ok(())
    }

    pub fn pop_input(&mut self) -> Option<InputEvent> {
        if self.event_len == 0 {
            return None;
        }
        let event = self.events[self.event_head].take();
        self.event_head = (self.event_head + 1) % MAX_EVENTS;
        self.event_len -= 1;
        event
    }

    pub fn render_to_framebuffer(&self, fb: &mut Framebuffer) {
        // Render desktop background
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: fb.width,
                height: fb.height,
            },
            Color::BLUE,
        );

        // Render visible windows
        for win in self.windows.iter().flatten() {
            if win.visible {
                let color = if win.focused {
                    Color::WHITE
                } else {
                    win.bg_color
                };
                fb.fill_rect(win.bounds, color);
            }
        }
    }

    pub const fn window_count(&self) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < MAX_WINDOWS {
            if self.windows[i].is_some() {
                count += 1;
            }
            i += 1;
        }
        count
    }
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compositor_creates_and_focuses_windows() {
        let mut c = Compositor::new();
        let a = c
            .create_window(Rect {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            })
            .unwrap();
        let b = c
            .create_window(Rect {
                x: 10,
                y: 10,
                width: 400,
                height: 300,
            })
            .unwrap();
        c.focus(b).unwrap();
        assert_eq!(c.window_count(), 2);
        assert!(
            c.windows
                .iter()
                .flatten()
                .find(|w| w.id == b)
                .unwrap()
                .focused
        );
        assert!(
            !c.windows
                .iter()
                .flatten()
                .find(|w| w.id == a)
                .unwrap()
                .focused
        );
    }

    #[test]
    fn terminal_and_notifications_work() {
        let mut term = TerminalBackend::new();
        term.write_char(b'A');
        term.write_char(b'W');
        term.write_char(b'E');
        assert_eq!(term.grid[0][0], b'A');
        assert_eq!(term.grid[0][1], b'W');
        assert_eq!(term.grid[0][2], b'E');

        let mut notif = NotificationDaemon::new();
        let nid = notif.push_notification(0xABCD, 100).unwrap();
        assert_eq!(nid, 1);
    }
}
