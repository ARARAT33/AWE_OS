//! Native AWEOS Text Editor Application.

#![no_std]

use awe_ayui::{AppType, Color, Compositor, Framebuffer, Rect};

pub struct TextEditorApp {
    pub window_width: u32,
    pub window_height: u32,
    pub document_buffer: [u8; 1024],
    pub document_len: usize,
}

impl TextEditorApp {
    pub fn new() -> Self {
        let mut app = Self {
            window_width: 600,
            window_height: 400,
            document_buffer: [0u8; 1024],
            document_len: 0,
        };
        app.set_text("Welcome to AWE_OS Text Editor v1.0\nType document contents here...");
        app
    }

    pub fn set_text(&mut self, text: &str) {
        let len = text.len().min(1024);
        self.document_buffer[..len].copy_from_slice(&text.as_bytes()[..len]);
        self.document_len = len;
    }

    pub fn launch(&self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 120,
            y: 80,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::TextEditor, b"Text Editor");
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
                r: 24,
                g: 28,
                b: 36,
                a: 255,
            },
        );

        // Toolbar
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height: 32,
            },
            Color {
                r: 36,
                g: 42,
                b: 54,
                a: 255,
            },
        );
        fb.draw_text(10, 8, "File", Color::WHITE);
        fb.draw_text(60, 8, "Edit", Color::WHITE);
        fb.draw_text(110, 8, "View", Color::WHITE);
        fb.draw_text(160, 8, "Help", Color::WHITE);

        // Document body
        let mut cur_y = 45i32;
        let mut line_start = 0usize;
        for (i, &b) in self.document_buffer[..self.document_len].iter().enumerate() {
            if b == b'\n' || i == self.document_len - 1 {
                let end = if b == b'\n' { i } else { i + 1 };
                if let Ok(line) = core::str::from_utf8(&self.document_buffer[line_start..end]) {
                    fb.draw_text(12, cur_y, line, Color::WHITE);
                    cur_y += 18;
                }
                line_start = i + 1;
            }
        }
    }
}

impl Default for TextEditorApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_editor_app() {
        let mut compositor = Compositor::new();
        let app = TextEditorApp::new();
        app.launch(&mut compositor);
        assert_eq!(compositor.window_count(), 1);

        extern crate alloc;
        use alloc::vec;
        let mut buf = vec![0u8; 600 * 400 * 4];
        app.render(&mut buf, 600, 400);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
