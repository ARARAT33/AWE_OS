//! Native AWEOS Package Center Application.

#![no_std]

use awe_ayui::{AppType, Color, Compositor, Framebuffer, Rect};

pub struct PackageCenterApp {
    pub window_width: u32,
    pub window_height: u32,
    pub installed_count: u32,
}

impl PackageCenterApp {
    pub fn new() -> Self {
        Self {
            window_width: 640,
            window_height: 480,
            installed_count: 12,
        }
    }

    pub fn launch(&self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 140,
            y: 90,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::PackageCenter, b"Package Center");
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
                r: 22,
                g: 26,
                b: 36,
                a: 255,
            },
        );

        fb.draw_text(20, 20, "AWE_OS Package & Driver Center", Color::WHITE);
        fb.draw_text(20, 50, "Verified .awos & .asd Store", Color { r: 160, g: 170, b: 190, a: 255 });

        let pkgs = [
            "terminal.awos (Official AWE Terminal)",
            "filemanager.awos (Universal Files)",
            "virtio_net.asd (VirtIO Network Driver)",
            "nvme.asd (NVMe Storage Driver)",
        ];

        for (i, pkg) in pkgs.iter().enumerate() {
            let y = 90 + (i as i32) * 48;
            fb.fill_rect(
                Rect {
                    x: 20,
                    y,
                    width: width.saturating_sub(40),
                    height: 38,
                },
                Color {
                    r: 36,
                    g: 42,
                    b: 56,
                    a: 255,
                },
            );
            fb.draw_text(30, y + 10, pkg, Color::WHITE);
        }
    }
}

impl Default for PackageCenterApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_center_app() {
        let mut compositor = Compositor::new();
        let app = PackageCenterApp::new();
        app.launch(&mut compositor);
        assert_eq!(compositor.window_count(), 1);

        extern crate alloc;
        use alloc::vec;
        let mut buf = vec![0u8; 640 * 480 * 4];
        app.render(&mut buf, 640, 480);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
