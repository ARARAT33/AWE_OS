#![no_std]

use awe_ayui::{AppType, Compositor, Rect};

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
            window_width: 500,
            window_height: 350,
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
    }
}
