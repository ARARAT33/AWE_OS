#![no_std]

use awe_ayui::{AppType, Compositor, Rect};

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
    }
}
