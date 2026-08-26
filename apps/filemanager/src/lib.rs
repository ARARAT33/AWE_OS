//! Native AWEOS Universal File Manager Application.

#![no_std]

use awe_ayui::{AppType, Color, Compositor, Framebuffer, Rect};

pub struct AweFileManagerApp {
    pub window_width: u32,
    pub window_height: u32,
    pub current_path: &'static str,
    pub entries: [&'static str; 5],
    pub awefs_mounted: bool,
}

impl AweFileManagerApp {
    pub fn new() -> Self {
        Self {
            window_width: 680,
            window_height: 440,
            current_path: "awefs://system/root",
            entries: [
                "System",
                "Applications",
                "User Documents",
                "Downloads",
                "Media",
            ],
            awefs_mounted: true,
        }
    }

    pub fn launch(&self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 80,
            y: 60,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::FileManager, b"File Manager");
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
            gpu_accel: false,
        };

        // Sidebar background
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height,
            },
            Color {
                r: 24,
                g: 28,
                b: 40,
                a: 255,
            },
        );

        // Sidebar title
        fb.draw_text(
            12,
            12,
            "PLACES",
            Color {
                r: 160,
                g: 170,
                b: 190,
                a: 255,
            },
        );
        fb.draw_text(12, 36, "Root Drive", Color::WHITE);
        fb.draw_text(12, 56, "Home", Color::WHITE);
        fb.draw_text(12, 76, "Network", Color::WHITE);

        // Main content area
        fb.fill_rect(
            Rect {
                x: 200,
                y: 0,
                width: width.saturating_sub(200),
                height,
            },
            Color {
                r: 32,
                g: 38,
                b: 54,
                a: 255,
            },
        );

        // Location bar
        fb.fill_rect(
            Rect {
                x: 210,
                y: 10,
                width: width.saturating_sub(220),
                height: 28,
            },
            Color {
                r: 45,
                g: 52,
                b: 70,
                a: 255,
            },
        );
        fb.draw_text(220, 18, self.current_path, Color::WHITE);
        if self.awefs_mounted {
            fb.draw_text((width as i32) - 150, 18, "[AWEFS Active]", Color::GREEN);
        }

        // Entries list
        for (i, entry) in self.entries.iter().enumerate() {
            let y = 60 + (i as i32) * 28;
            fb.draw_text(
                220,
                y,
                "[DIR] ",
                Color {
                    r: 0,
                    g: 153,
                    b: 255,
                    a: 255,
                },
            );
            fb.draw_text(270, y, entry, Color::WHITE);
        }
    }
}

impl Default for AweFileManagerApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_manager_app() {
        let mut compositor = Compositor::new();
        let app = AweFileManagerApp::new();
        app.launch(&mut compositor);
        assert_eq!(compositor.window_count(), 1);

        extern crate alloc;
        use alloc::vec;
        let mut buf = vec![0u8; 800 * 600 * 4];
        app.render(&mut buf, 800, 600);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
