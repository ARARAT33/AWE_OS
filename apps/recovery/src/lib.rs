#![no_std]

use awe_ayui::{AppType, Compositor, Rect};

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
    }
}
