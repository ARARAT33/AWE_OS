//! Native AWEOS Calculator Application.

#![no_std]

use awe_ayui::{AppType, Color, Compositor, Framebuffer, Rect};

pub struct AweCalculatorApp {
    pub window_width: u32,
    pub window_height: u32,
    pub display_buffer: [u8; 16],
    pub display_len: usize,
}

impl AweCalculatorApp {
    pub fn new() -> Self {
        let mut app = Self {
            window_width: 320,
            window_height: 420,
            display_buffer: [0u8; 16],
            display_len: 0,
        };
        app.set_display("0");
        app
    }

    pub fn set_display(&mut self, val: &str) {
        let len = val.len().min(16);
        self.display_buffer[..len].copy_from_slice(&val.as_bytes()[..len]);
        self.display_len = len;
    }

    pub fn launch(&self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 200,
            y: 150,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::Generic, b"Calculator");
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
            gpu_accel: false,
        };

        // Window background
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            Color {
                r: 28,
                g: 32,
                b: 42,
                a: 255,
            },
        );

        // Display box
        fb.fill_rect(
            Rect {
                x: 10,
                y: 10,
                width: width.saturating_sub(20),
                height: 50,
            },
            Color {
                r: 18,
                g: 20,
                b: 28,
                a: 255,
            },
        );

        let disp_str =
            core::str::from_utf8(&self.display_buffer[..self.display_len]).unwrap_or("0");
        fb.draw_text(20, 26, disp_str, Color::WHITE);

        // Keypad buttons
        let keys = [
            ["C", "(", ")", "/"],
            ["7", "8", "9", "*"],
            ["4", "5", "6", "-"],
            ["1", "2", "3", "+"],
            ["0", ".", "C", "="],
        ];

        for (r, row) in keys.iter().enumerate() {
            for (c, key) in row.iter().enumerate() {
                let bx = 10 + (c as i32) * 72;
                let by = 70 + (r as i32) * 64;
                fb.fill_rect(
                    Rect {
                        x: bx,
                        y: by,
                        width: 64,
                        height: 56,
                    },
                    Color {
                        r: 45,
                        g: 52,
                        b: 68,
                        a: 255,
                    },
                );
                fb.draw_text(bx + 26, by + 20, key, Color::WHITE);
            }
        }
    }
}

impl Default for AweCalculatorApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_app() {
        let mut compositor = Compositor::new();
        let app = AweCalculatorApp::new();
        app.launch(&mut compositor);
        assert_eq!(compositor.window_count(), 1);

        extern crate alloc;
        use alloc::vec;
        let mut buf = vec![0u8; 320 * 420 * 4];
        app.render(&mut buf, 320, 420);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
