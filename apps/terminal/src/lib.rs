//! Native AWEOS Terminal Application.

use awe_ayui::{Color, Framebuffer, Rect, TerminalBackend};

pub struct AweTerminalApp {
    pub backend: TerminalBackend,
    pub title: &'static str,
}

impl AweTerminalApp {
    pub fn new() -> Self {
        Self {
            backend: TerminalBackend::new(),
            title: "AWEOS GPU-Accelerated Terminal",
        }
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
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
    fn test_terminal_app_render() {
        let app = AweTerminalApp::new();
        let mut buf = vec![0u8; 640 * 480 * 4];
        app.render(&mut buf, 640, 480);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
