//! Bounded AYUI desktop compositor, software rendering engine, widget system,
//! desktop shell, terminal emulator backend, multi-monitor, clipboard, accessibility,
//! notifications, themes, GPU path, and process execution stack.

#![no_std]

pub const MAX_WINDOWS: usize = 64;
pub const MAX_EVENTS: usize = 128;
pub const MAX_WIDTH: u32 = 16_384;
pub const MAX_HEIGHT: u32 = 16_384;
pub const MAX_NOTIFICATIONS: usize = 16;
pub const MAX_MONITORS: usize = 4;
pub const MAX_CLIPBOARD_LEN: usize = 1024;
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
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
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
    pub const LIGHT_GRAY: Self = Self {
        r: 192,
        g: 192,
        b: 192,
        a: 255,
    };
    pub const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppType {
    Launcher,
    Shell,
    Terminal,
    FileManager,
    Settings,
    SystemMonitor,
    TextEditor,
    PackageCenter,
    UpdateCenter,
    Recovery,
    Generic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub id: WindowId,
    pub bounds: Rect,
    pub visible: bool,
    pub focused: bool,
    pub bg_color: Color,
    pub app_type: AppType,
    pub title_len: usize,
    pub title_bytes: [u8; 32],
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
    BufferTooSmall,
}

/// Theme settings for AYUI
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub bg_color: Color,
    pub window_border: Color,
    pub titlebar_active: Color,
    pub titlebar_inactive: Color,
    pub text_color: Color,
    pub accent_color: Color,
    pub dark_mode: bool,
}

impl Theme {
    pub const fn default_dark() -> Self {
        Self {
            bg_color: Color {
                r: 24,
                g: 28,
                b: 36,
                a: 255,
            },
            window_border: Color {
                r: 60,
                g: 64,
                b: 72,
                a: 255,
            },
            titlebar_active: Color {
                r: 0,
                g: 120,
                b: 215,
                a: 255,
            },
            titlebar_inactive: Color {
                r: 45,
                g: 50,
                b: 60,
                a: 255,
            },
            text_color: Color::WHITE,
            accent_color: Color {
                r: 0,
                g: 153,
                b: 255,
                a: 255,
            },
            dark_mode: true,
        }
    }
}

/// Accessibility Settings
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accessibility {
    pub high_contrast: bool,
    pub large_text: bool,
    pub screen_reader_active: bool,
    pub sticky_keys: bool,
}

impl Accessibility {
    pub const fn new() -> Self {
        Self {
            high_contrast: false,
            large_text: false,
            screen_reader_active: false,
            sticky_keys: false,
        }
    }
}

impl Default for Accessibility {
    fn default() -> Self {
        Self::new()
    }
}

/// Clipboard Manager
pub struct Clipboard {
    data: [u8; MAX_CLIPBOARD_LEN],
    len: usize,
}

impl Clipboard {
    pub const fn new() -> Self {
        Self {
            data: [0; MAX_CLIPBOARD_LEN],
            len: 0,
        }
    }

    pub fn copy(&mut self, src: &[u8]) -> Result<(), UiError> {
        if src.len() > MAX_CLIPBOARD_LEN {
            return Err(UiError::BufferTooSmall);
        }
        self.data[..src.len()].copy_from_slice(src);
        self.len = src.len();
        Ok(())
    }

    pub fn paste<'a>(&'a self, dst: &'a mut [u8]) -> usize {
        let copy_len = self.len.min(dst.len());
        dst[..copy_len].copy_from_slice(&self.data[..copy_len]);
        copy_len
    }

    pub fn text(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Display Monitor Info for Multi-Monitor Architecture
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Monitor {
    pub id: u8,
    pub bounds: Rect,
    pub primary: bool,
    pub gpu_accelerated: bool,
}

/// 8x8 Bitmap Font table covering ASCII characters 0x20 (' ') through 0x7E ('~').
pub static FONT_8X8: [[u8; 8]; 95] = [
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // ' ' (32)
    [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00], // '!' (33)
    [0x66, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00], // '"' (34)
    [0x66, 0x66, 0xFF, 0x66, 0xFF, 0x66, 0x66, 0x00], // '#' (35)
    [0x18, 0x7E, 0x18, 0x3C, 0x18, 0x7E, 0x18, 0x00], // '$' (36)
    [0x00, 0x63, 0x6C, 0x18, 0x30, 0x66, 0x43, 0x00], // '%' (37)
    [0x38, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0x76, 0x00], // '&' (38)
    [0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00], // '\'' (39)
    [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00], // '(' (40)
    [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00], // ')' (41)
    [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00], // '*' (42)
    [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00], // '+' (43)
    [0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30, 0x00], // ',' (44)
    [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00], // '-' (45)
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00], // '.' (46)
    [0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x80, 0x00], // '/' (47)
    [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00], // '0' (48)
    [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00], // '1' (49)
    [0x3C, 0x66, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00], // '2' (50)
    [0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00], // '3' (51)
    [0x0C, 0x1C, 0x3C, 0x6C, 0xFE, 0x0C, 0x0C, 0x00], // '4' (52)
    [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00], // '5' (53)
    [0x3C, 0x66, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0x00], // '6' (54)
    [0x7E, 0x66, 0x0C, 0x18, 0x18, 0x18, 0x18, 0x00], // '7' (55)
    [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00], // '8' (56)
    [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x66, 0x3C, 0x00], // '9' (57)
    [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00], // ':' (58)
    [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x30, 0x00], // ';' (59)
    [0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00], // '<' (60)
    [0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00], // '=' (61)
    [0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x00], // '>' (62)
    [0x3C, 0x66, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00], // '?' (63)
    [0x3C, 0x66, 0x6E, 0x6E, 0x60, 0x62, 0x3C, 0x00], // '@' (64)
    [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00], // 'A' (65)
    [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00], // 'B' (66)
    [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00], // 'C' (67)
    [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00], // 'D' (68)
    [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x7E, 0x00], // 'E' (69)
    [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x60, 0x00], // 'F' (70)
    [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3E, 0x00], // 'G' (71)
    [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00], // 'H' (72)
    [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'I' (73)
    [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x6C, 0x38, 0x00], // 'J' (74)
    [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00], // 'K' (75)
    [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00], // 'L' (76)
    [0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x00], // 'M' (77)
    [0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x00], // 'N' (78)
    [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // 'O' (79)
    [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00], // 'P' (80)
    [0x3C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x0E, 0x00], // 'Q' (81)
    [0x7C, 0x66, 0x66, 0x7C, 0x70, 0x68, 0x66, 0x00], // 'R' (82)
    [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00], // 'S' (83)
    [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // 'T' (84)
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // 'U' (85)
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00], // 'V' (86)
    [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00], // 'W' (87)
    [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00], // 'X' (88)
    [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00], // 'Y' (89)
    [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00], // 'Z' (90)
    [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00], // '[' (91)
    [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x00], // '\' (92)
    [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00], // ']' (93)
    [0x18, 0x3C, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00], // '^' (94)
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00], // '_' (95)
    [0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00], // '`' (96)
    [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00], // 'a' (97)
    [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x00], // 'b' (98)
    [0x00, 0x00, 0x3C, 0x66, 0x60, 0x66, 0x3C, 0x00], // 'c' (99)
    [0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x3E, 0x00], // 'd' (100)
    [0x00, 0x00, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00], // 'e' (101)
    [0x1C, 0x30, 0x7C, 0x30, 0x30, 0x30, 0x30, 0x00], // 'f' (102)
    [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x7C], // 'g' (103)
    [0x60, 0x60, 0x6C, 0x76, 0x66, 0x66, 0x66, 0x00], // 'h' (104)
    [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'i' (105)
    [0x06, 0x00, 0x0E, 0x06, 0x06, 0x06, 0x66, 0x3C], // 'j' (106)
    [0x60, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0x00], // 'k' (107)
    [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'l' (108)
    [0x00, 0x00, 0x66, 0x7F, 0x7F, 0x6B, 0x63, 0x00], // 'm' (109)
    [0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00], // 'n' (110)
    [0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x00], // 'o' (111)
    [0x00, 0x00, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60], // 'p' (112)
    [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x06], // 'q' (113)
    [0x00, 0x00, 0x6C, 0x76, 0x60, 0x60, 0x60, 0x00], // 'r' (114)
    [0x00, 0x00, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0x00], // 's' (115)
    [0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x0E, 0x00], // 't' (116)
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x3E, 0x00], // 'u' (117)
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00], // 'v' (118)
    [0x00, 0x00, 0x63, 0x6B, 0x7F, 0x3E, 0x36, 0x00], // 'w' (119)
    [0x00, 0x00, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x00], // 'x' (120)
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x7C], // 'y' (121)
    [0x00, 0x00, 0x7E, 0x0C, 0x18, 0x30, 0x7E, 0x00], // 'z' (122)
    [0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0x00], // '{' (123)
    [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // '|' (124)
    [0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0x00], // '}' (125)
    [0x32, 0x4C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // '~' (126)
];

/// Returns the 8x8 font glyph row bitmasks for ASCII character `ch`.
pub fn get_glyph(ch: u8) -> &'static [u8; 8] {
    if (32..=126).contains(&ch) {
        &FONT_8X8[(ch - 32) as usize]
    } else {
        &FONT_8X8[63 - 32] // Fallback '?'
    }
}

/// Software / GPU Framebuffer for display composition.
#[derive(Debug)]
pub struct Framebuffer<'a> {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub buffer: &'a mut [u8],
    pub gpu_accel: bool,
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

    /// Renders a single character glyph using the 8x8 bitmap font table into the framebuffer.
    pub fn draw_char(&mut self, x: i32, y: i32, ch: u8, color: Color) {
        let glyph = get_glyph(ch);
        for row in 0..8i32 {
            let row_mask = glyph[row as usize];
            if row_mask == 0 {
                continue;
            }
            let py = y + row;
            if py < 0 || (py as u32) >= self.height {
                continue;
            }
            for col in 0..8i32 {
                if (row_mask & (1 << (7 - col))) != 0 {
                    let px = x + col;
                    if px >= 0 && (px as u32) < self.width {
                        let offset = (((py as u32) * self.stride + (px as u32)) * 4) as usize;
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
    }

    /// Renders text with basic 8x8 font representation into the framebuffer.
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Color) {
        let mut cur_x = x;
        for byte in text.bytes() {
            if byte != b' ' {
                self.draw_char(cur_x, y, byte, color);
            }
            cur_x += 9;
        }
    }

    /// Draws a mouse cursor
    pub fn draw_cursor(&mut self, x: i32, y: i32, color: Color) {
        let cursor_rect = Rect {
            x,
            y,
            width: 10,
            height: 10,
        };
        self.fill_rect(cursor_rect, color);
    }
}

/// Widget system primitives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetType {
    Button { label_id: u16 },
    Label { text_len: u8 },
    Panel,
    TextField,
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

/// AYUI Compositor and Window Manager
pub struct Compositor {
    windows: [Option<Window>; MAX_WINDOWS],
    events: [Option<InputEvent>; MAX_EVENTS],
    event_head: usize,
    event_len: usize,
    next_id: u16,
    monitors: [Option<Monitor>; MAX_MONITORS],
    monitor_count: usize,
    pub theme: Theme,
    pub accessibility: Accessibility,
    pub clipboard: Clipboard,
    pub pointer_x: i32,
    pub pointer_y: i32,
    pub pointer_buttons: u8,
}

impl Compositor {
    pub const fn new() -> Self {
        Self {
            windows: [None; MAX_WINDOWS],
            events: [None; MAX_EVENTS],
            event_head: 0,
            event_len: 0,
            next_id: 1,
            monitors: [None; MAX_MONITORS],
            monitor_count: 0,
            theme: Theme::default_dark(),
            accessibility: Accessibility::new(),
            clipboard: Clipboard::new(),
            pointer_x: 400,
            pointer_y: 300,
            pointer_buttons: 0,
        }
    }

    pub fn register_monitor(
        &mut self,
        bounds: Rect,
        primary: bool,
        gpu_accel: bool,
    ) -> Result<u8, UiError> {
        if self.monitor_count >= MAX_MONITORS {
            return Err(UiError::Full);
        }
        let id = self.monitor_count as u8;
        self.monitors[self.monitor_count] = Some(Monitor {
            id,
            bounds,
            primary,
            gpu_accelerated: gpu_accel,
        });
        self.monitor_count += 1;
        Ok(id)
    }

    pub fn create_window(&mut self, bounds: Rect) -> Result<WindowId, UiError> {
        self.create_app_window(bounds, AppType::Generic, b"Window")
    }

    pub fn create_app_window(
        &mut self,
        bounds: Rect,
        app_type: AppType,
        title: &[u8],
    ) -> Result<WindowId, UiError> {
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

        let mut title_bytes = [0u8; 32];
        let title_len = title.len().min(32);
        title_bytes[..title_len].copy_from_slice(&title[..title_len]);

        self.windows[slot] = Some(Window {
            id,
            bounds,
            visible: true,
            focused: false,
            bg_color: self.theme.bg_color,
            app_type,
            title_len,
            title_bytes,
        });
        self.focus(id)?;
        Ok(id)
    }

    pub fn destroy_window(&mut self, id: WindowId) -> Result<(), UiError> {
        for win in self.windows.iter_mut() {
            if let Some(w) = win
                && w.id == id
            {
                *win = None;
                return Ok(());
            }
        }
        Err(UiError::InvalidWindow)
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
        match event {
            InputEvent::Pointer { x, y, buttons } => {
                if x < -1_048_576 || y < -1_048_576 || x > 1_048_576 || y > 1_048_576 {
                    return Err(UiError::InvalidEvent);
                }
                self.pointer_x = x;
                self.pointer_y = y;
                self.pointer_buttons = buttons;

                // Handle window focus on mouse click
                if buttons & 1 != 0 {
                    for win in self.windows.iter().flatten() {
                        if win.visible && win.bounds.contains(x, y) {
                            let wid = win.id;
                            let _ = self.focus(wid);
                            break;
                        }
                    }
                }
            }
            InputEvent::Key { .. } => {}
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
        // Render desktop wallpaper background
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: fb.width,
                height: fb.height,
            },
            self.theme.bg_color,
        );

        // Render Desktop Taskbar / Panel
        let taskbar_rect = Rect {
            x: 0,
            y: (fb.height as i32) - 40,
            width: fb.width,
            height: 40,
        };
        fb.fill_rect(
            taskbar_rect,
            Color {
                r: 18,
                g: 20,
                b: 26,
                a: 255,
            },
        );
        fb.draw_text(
            10,
            (fb.height as i32) - 30,
            "AWE_OS AYUI Desktop",
            self.theme.text_color,
        );

        // Render visible windows
        for win in self.windows.iter().flatten() {
            if win.visible {
                // Window Title Bar
                let titlebar_color = if win.focused {
                    self.theme.titlebar_active
                } else {
                    self.theme.titlebar_inactive
                };
                let title_rect = Rect {
                    x: win.bounds.x,
                    y: win.bounds.y,
                    width: win.bounds.width,
                    height: 24,
                };
                fb.fill_rect(title_rect, titlebar_color);

                // Window Content Body
                let content_rect = Rect {
                    x: win.bounds.x,
                    y: win.bounds.y + 24,
                    width: win.bounds.width,
                    height: win.bounds.height.saturating_sub(24),
                };
                fb.fill_rect(content_rect, win.bg_color);

                // Render App Title
                let title_str =
                    core::str::from_utf8(&win.title_bytes[..win.title_len]).unwrap_or("App");
                fb.draw_text(win.bounds.x + 8, win.bounds.y + 6, title_str, Color::WHITE);
            }
        }

        // Render Pointer Cursor
        fb.draw_cursor(self.pointer_x, self.pointer_y, Color::YELLOW);
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

    #[test]
    fn multi_monitor_and_clipboard_work() {
        let mut c = Compositor::new();
        let mid = c
            .register_monitor(
                Rect {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                true,
                true,
            )
            .unwrap();
        assert_eq!(mid, 0);

        c.clipboard.copy(b"AWEOS Clipboard").unwrap();
        assert_eq!(c.clipboard.text(), b"AWEOS Clipboard");
    }

    #[test]
    fn font_rendering_and_glyph_drawing_work() {
        let glyph_a = get_glyph(b'A');
        assert_eq!(*glyph_a, [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00]);

        let glyph_lower_a = get_glyph(b'a');
        assert_eq!(
            *glyph_lower_a,
            [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00]
        );

        let glyph_num_5 = get_glyph(b'5');
        assert_eq!(
            *glyph_num_5,
            [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00]
        );

        let glyph_symbol = get_glyph(b'>');
        assert_eq!(
            *glyph_symbol,
            [0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x00]
        );

        let mut buf = [0u8; 100 * 100 * 4];
        let mut fb = Framebuffer {
            width: 100,
            height: 100,
            stride: 100,
            buffer: &mut buf,
            gpu_accel: false,
        };

        fb.draw_text(10, 10, "AWEOS 2026!", Color::WHITE);

        // Check that pixels were rendered for 'A' at offset (10, 10)
        // 'A' row 0 has bits 0x18 at cols 3 and 4 -> x = 13, 14, y = 10
        let offset_13_10 = ((10 * 100 + 13) * 4) as usize;
        assert_eq!(buf[offset_13_10], 255); // Color::WHITE b = 255
    }
}
