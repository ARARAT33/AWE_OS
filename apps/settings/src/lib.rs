//! Native AWEOS System Settings Application.

use awe_ayui::{Color, Framebuffer, Rect};

pub struct AweSettingsApp {
    pub categories: [&'static str; 6],
    pub active_category: usize,
}

impl AweSettingsApp {
    pub fn new() -> Self {
        Self {
            categories: [
                "System & Hardware",
                "Display & Sound",
                "Network & Wi-Fi",
                "Security & Privacy",
                "Continuum Mesh",
                "Updates & Recovery",
            ],
            active_category: 0,
        }
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
        };

        // Background
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            Color {
                r: 20,
                g: 22,
                b: 32,
                a: 255,
            },
        );
    }
}

impl Default for AweSettingsApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_app() {
        let app = AweSettingsApp::new();
        assert_eq!(app.categories.len(), 6);
        let mut buf = vec![0u8; 800 * 600 * 4];
        app.render(&mut buf, 800, 600);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
