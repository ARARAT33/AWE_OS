//! Native AWEOS Terminal Application.

#![no_std]

use awe_ayui::{
    AppType, Color, Compositor, Framebuffer, Rect, TERM_COLS, TERM_ROWS, TerminalBackend,
};

pub struct AweTerminalApp {
    pub backend: TerminalBackend,
    pub title: &'static str,
    pub window_width: u32,
    pub window_height: u32,
    pub active: bool,
    pub command_buffer: [u8; 128],
    pub command_len: usize,
}

impl AweTerminalApp {
    pub fn new() -> Self {
        let mut app = Self {
            backend: TerminalBackend::new(),
            title: "AWEOS GPU-Accelerated Terminal",
            window_width: 720,
            window_height: 450,
            active: false,
            command_buffer: [0u8; 128],
            command_len: 0,
        };
        app.write_str("AWEOS Universal Singularity Terminal v1.0.0\nType 'help' for available commands.\n\naweos> ");
        app
    }

    pub fn write_str(&mut self, text: &str) {
        for b in text.bytes() {
            self.backend.write_char(b);
        }
    }

    pub fn launch(&mut self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 60,
            y: 40,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::Terminal, b"AWEOS Terminal");
        self.active = true;
    }

    pub fn handle_char(&mut self, ch: u8) {
        match ch {
            b'\r' | b'\n' => {
                self.execute_command();
            }
            8 | 127 => {
                if self.command_len > 0 {
                    self.command_len -= 1;
                    if self.backend.cursor_col > 0 {
                        self.backend.cursor_col -= 1;
                        self.backend.grid[self.backend.cursor_row][self.backend.cursor_col] = b' ';
                    }
                }
            }
            c if (32..=126).contains(&c) && self.command_len < self.command_buffer.len() => {
                self.command_buffer[self.command_len] = c;
                self.command_len += 1;
                self.backend.write_char(c);
            }
            _ => {}
        }
    }

    pub fn execute_command(&mut self) {
        self.backend.write_char(b'\n');
        let cmd_bytes = &self.command_buffer[..self.command_len];
        if !cmd_bytes.is_empty() {
            if cmd_bytes == b"help" {
                self.write_str(
                    "Available commands: help, ver, sysinfo, uname, clear, status, echo <msg>\n",
                );
            } else if cmd_bytes == b"ver" {
                self.write_str("AWEOS Universal Singularity v2.5.0 (CellKernel x86_64)\n");
            } else if cmd_bytes == b"sysinfo" {
                self.write_str(
                    "CPU: x86_64 Native | Memory: 4096MB | Kernel: CellKernel | Compositor: AYUI\n",
                );
            } else if cmd_bytes == b"uname" {
                self.write_str("AWEOS 2.5.0 CellKernel x86_64\n");
            } else if cmd_bytes == b"status" {
                self.write_str("System status: RUNNING | AYUI Compositor: ACTIVE | Services: OK\n");
            } else if cmd_bytes == b"clear" {
                self.backend.grid = [[b' '; TERM_COLS]; TERM_ROWS];
                self.backend.cursor_row = 0;
                self.backend.cursor_col = 0;
            } else if cmd_bytes.starts_with(b"echo ") {
                for &b in &cmd_bytes[5..] {
                    self.backend.write_char(b);
                }
                self.backend.write_char(b'\n');
            } else {
                self.write_str("Command not found. Type 'help' for available commands.\n");
            }
        }
        self.command_len = 0;
        self.write_str("aweos> ");
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
            gpu_accel: false,
        };

        // Dark background
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            Color {
                r: 12,
                g: 14,
                b: 20,
                a: 255,
            },
        );

        // Header
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height: 30,
            },
            Color {
                r: 30,
                g: 36,
                b: 48,
                a: 255,
            },
        );
        fb.draw_text(12, 8, self.title, Color::WHITE);

        // Render Terminal Grid
        let text_color = Color {
            r: 80,
            g: 220,
            b: 120,
            a: 255,
        };

        for r in 0..TERM_ROWS {
            let row_y = 38 + (r as i32) * 16;
            if row_y + 12 > height as i32 {
                break;
            }
            for c in 0..TERM_COLS {
                let col_x = 10 + (c as i32) * 9;
                if col_x + 8 > width as i32 {
                    break;
                }
                let ch = self.backend.grid[r][c];
                if ch != b' ' {
                    fb.draw_char(col_x, row_y, ch, text_color);
                }
            }
        }

        // Render Cursor
        let cursor_x = 10 + (self.backend.cursor_col as i32) * 9;
        let cursor_y = 38 + (self.backend.cursor_row as i32) * 16;
        if cursor_x + 8 <= width as i32 && cursor_y + 14 <= height as i32 {
            fb.draw_char(cursor_x, cursor_y, b'_', text_color);
        }
    }
}

impl Default for AweTerminalApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_app_lifecycle_and_commands() {
        let mut compositor = Compositor::new();
        let mut app = AweTerminalApp::new();
        app.launch(&mut compositor);
        assert!(app.active);
        assert_eq!(compositor.window_count(), 1);

        for &b in b"help\n" {
            app.handle_char(b);
        }

        let mut buf = [0u8; 640 * 480 * 4];
        app.render(&mut buf, 640, 480);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
