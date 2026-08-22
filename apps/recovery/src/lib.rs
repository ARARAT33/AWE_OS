//! Native AWEOS Recovery Environment Application.

#![no_std]

use awe_ayui::{AppType, Color, Compositor, Framebuffer, Rect};

pub struct RecoveryEnvApp {
    pub window_width: u32,
    pub window_height: u32,
    pub system_healthy: bool,
}

impl RecoveryEnvApp {
    pub fn new() -> Self {
        Self {
            window_width: 700,
            window_height: 500,
            system_healthy: true,
        }
    }

    pub fn launch(&self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 50,
            y: 50,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::Recovery, b"Recovery Environment");
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
            gpu_accel: false,
        };

        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            Color {
                r: 28,
                g: 18,
                b: 22,
                a: 255,
            },
        );

        fb.draw_text(20, 20, "AWE_OS Atomic Recovery & Maintenance", Color::WHITE);
        fb.draw_text(
            20,
            50,
            "Status: HEALTHY (Slot A Active)",
            Color {
                r: 80,
                g: 220,
                b: 120,
                a: 255,
            },
        );

        let actions = [
            "1. Rollback to Slot B Backup",
            "2. Offline Filesystem FSCK Repair",
            "3. Restore Factory System Image",
            "4. Reboot into Normal Boot Mode",
        ];

        for (i, action) in actions.iter().enumerate() {
            let y = 100 + (i as i32) * 50;
            fb.fill_rect(
                Rect {
                    x: 20,
                    y,
                    width: width.saturating_sub(40),
                    height: 40,
                },
                Color {
                    r: 48,
                    g: 28,
                    b: 36,
                    a: 255,
                },
            );
            fb.draw_text(35, y + 12, action, Color::WHITE);
        }
    }
}

impl Default for RecoveryEnvApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_env_app() {
        let mut compositor = Compositor::new();
        let app = RecoveryEnvApp::new();
        app.launch(&mut compositor);
        assert_eq!(compositor.window_count(), 1);

        extern crate alloc;
        use alloc::vec;
        let mut buf = vec![0u8; 700 * 500 * 4];
        app.render(&mut buf, 700, 500);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
