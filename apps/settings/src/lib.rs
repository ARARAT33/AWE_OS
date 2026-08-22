//! Native AWEOS System Settings Application.

#![no_std]

use awe_ayui::{AppType, Color, Compositor, Framebuffer, Rect};

pub struct AweSettingsApp {
    pub window_width: u32,
    pub window_height: u32,
    pub categories: [&'static str; 6],
    pub active_category: usize,
}

impl AweSettingsApp {
    pub fn new() -> Self {
        Self {
            window_width: 700,
            window_height: 480,
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

    pub fn launch(&self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 90,
            y: 70,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::Settings, b"Settings");
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
            gpu_accel: false,
        };

        // Sidebar
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: 220,
                height,
            },
            Color {
                r: 22,
                g: 26,
                b: 36,
                a: 255,
            },
        );

        for (i, cat) in self.categories.iter().enumerate() {
            let y = 20 + (i as i32) * 36;
            let is_active = i == self.active_category;
            let bg = if is_active {
                Color {
                    r: 0,
                    g: 120,
                    b: 215,
                    a: 255,
                }
            } else {
                Color {
                    r: 22,
                    g: 26,
                    b: 36,
                    a: 255,
                }
            };
            fb.fill_rect(
                Rect {
                    x: 10,
                    y: y - 6,
                    width: 200,
                    height: 28,
                },
                bg,
            );
            fb.draw_text(20, y, cat, Color::WHITE);
        }

        // Content panel
        fb.fill_rect(
            Rect {
                x: 220,
                y: 0,
                width: width.saturating_sub(220),
                height,
            },
            Color {
                r: 30,
                g: 34,
                b: 46,
                a: 255,
            },
        );

        let active_name = self.categories[self.active_category];
        fb.draw_text(240, 20, active_name, Color::WHITE);
        fb.draw_text(
            240,
            50,
            "AWE_OS 1.0 Production Kernel & Service Stack",
            Color {
                r: 180,
                g: 190,
                b: 200,
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
        let mut compositor = Compositor::new();
        let app = AweSettingsApp::new();
        app.launch(&mut compositor);
        assert_eq!(compositor.window_count(), 1);

        extern crate alloc;
        use alloc::vec;
        let mut buf = vec![0u8; 800 * 600 * 4];
        app.render(&mut buf, 800, 600);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
