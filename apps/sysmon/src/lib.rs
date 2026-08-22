//! Native AWEOS System Monitor Application.

#![no_std]

use awe_ayui::{AppType, Color, Compositor, Framebuffer, Rect};

pub struct SysMonApp {
    pub window_width: u32,
    pub window_height: u32,
    pub cpu_usage: u8,
    pub memory_used_mb: u32,
    pub memory_total_mb: u32,
}

impl SysMonApp {
    pub fn new() -> Self {
        Self {
            window_width: 580,
            window_height: 380,
            cpu_usage: 12,
            memory_used_mb: 256,
            memory_total_mb: 4096,
        }
    }

    pub fn launch(&self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 100,
            y: 100,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::SystemMonitor, b"System Monitor");
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
                r: 20,
                g: 24,
                b: 34,
                a: 255,
            },
        );

        fb.draw_text(20, 20, "AWE_OS CellKernel Resource Monitor", Color::WHITE);

        // CPU Bar
        fb.draw_text(
            20,
            60,
            "CPU Usage: 12%",
            Color {
                r: 0,
                g: 153,
                b: 255,
                a: 255,
            },
        );
        fb.fill_rect(
            Rect {
                x: 20,
                y: 80,
                width: 500,
                height: 20,
            },
            Color {
                r: 40,
                g: 48,
                b: 64,
                a: 255,
            },
        );
        fb.fill_rect(
            Rect {
                x: 20,
                y: 80,
                width: 60, // 12% of 500
                height: 20,
            },
            Color {
                r: 0,
                g: 153,
                b: 255,
                a: 255,
            },
        );

        // Memory Bar
        fb.draw_text(
            20,
            120,
            "Memory Usage: 256MB / 4096MB",
            Color {
                r: 0,
                g: 255,
                b: 120,
                a: 255,
            },
        );
        fb.fill_rect(
            Rect {
                x: 20,
                y: 140,
                width: 500,
                height: 20,
            },
            Color {
                r: 40,
                g: 48,
                b: 64,
                a: 255,
            },
        );
        fb.fill_rect(
            Rect {
                x: 20,
                y: 140,
                width: 31, // (256/4096)*500
                height: 20,
            },
            Color {
                r: 0,
                g: 255,
                b: 120,
                a: 255,
            },
        );
    }
}

impl Default for SysMonApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sysmon_app() {
        let mut compositor = Compositor::new();
        let app = SysMonApp::new();
        app.launch(&mut compositor);
        assert_eq!(compositor.window_count(), 1);

        extern crate alloc;
        use alloc::vec;
        let mut buf = vec![0u8; 580 * 380 * 4];
        app.render(&mut buf, 580, 380);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
