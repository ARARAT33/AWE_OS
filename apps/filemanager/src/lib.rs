//! Native AWEOS Universal File Manager Application.

use awe_ayui::{Color, Framebuffer, Rect};

pub struct AweFileManagerApp {
    pub current_path: &'static str,
    pub entries: [&'static str; 5],
}

impl AweFileManagerApp {
    pub fn new() -> Self {
        Self {
            current_path: "awe://storage/root",
            entries: [
                "System",
                "Applications",
                "User Documents",
                "Downloads",
                "Media",
            ],
        }
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
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
        let app = AweFileManagerApp::new();
        assert_eq!(app.entries.len(), 5);
        let mut buf = vec![0u8; 800 * 600 * 4];
        app.render(&mut buf, 800, 600);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
