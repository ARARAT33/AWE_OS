#![no_std]

use awe_ayui::{AppType, Compositor, Framebuffer};
use super::{FramebufferInfo, RuntimeRect};

pub const MAX_DESKTOP_APPS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopApp { Terminal, FileManager, Settings, SystemMonitor, Calculator, TextEditor, PackageCenter, Recovery }

impl DesktopApp {
    pub const fn id(self) -> u8 {
        match self { Self::Terminal => 1, Self::FileManager => 2, Self::Settings => 3, Self::SystemMonitor => 4, Self::Calculator => 5, Self::TextEditor => 6, Self::PackageCenter => 7, Self::Recovery => 8 }
    }
    pub const fn app_type(self) -> AppType {
        match self { Self::Terminal => AppType::Terminal, Self::FileManager => AppType::FileManager, Self::Settings => AppType::Settings, Self::SystemMonitor => AppType::SystemMonitor, _ => AppType::Generic }
    }
    pub const fn title(self) -> &'static [u8] {
        match self { Self::Terminal => b"AWETerminal", Self::FileManager => b"AWE File Manager", Self::Settings => b"AWE Settings", Self::SystemMonitor => b"AWE System Monitor", Self::Calculator => b"AWE Calculator", Self::TextEditor => b"AWE Text Editor", Self::PackageCenter => b"AWE Package Center", Self::Recovery => b"AWE Recovery" }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopError { Full, Window, InvalidFramebuffer }

pub struct DesktopShell {
    pub compositor: Compositor,
    pub framebuffer: Option<FramebufferInfo>,
    windows: [Option<(DesktopApp, u16)>; MAX_DESKTOP_APPS],
    window_count: usize,
    pub clock_ticks: u64,
}

impl DesktopShell {
    pub const fn new() -> Self { Self { compositor: Compositor::new(), framebuffer: None, windows: [None; MAX_DESKTOP_APPS], window_count: 0, clock_ticks: 0 } }

    pub fn attach_framebuffer(&mut self, info: FramebufferInfo) -> Result<(), DesktopError> {
        if !info.validate() { return Err(DesktopError::InvalidFramebuffer); }
        self.framebuffer = Some(info);
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), DesktopError> {
        self.launch(DesktopApp::Terminal, RuntimeRect { x: 40, y: 50, width: 640, height: 420 })?;
        self.launch(DesktopApp::SystemMonitor, RuntimeRect { x: 150, y: 100, width: 440, height: 300 })?;
        Ok(())
    }

    pub fn launch(&mut self, app: DesktopApp, rect: RuntimeRect) -> Result<u16, DesktopError> {
        if self.windows.iter().flatten().any(|(a, _)| *a == app) { return Err(DesktopError::Window); }
        if self.window_count == MAX_DESKTOP_APPS { return Err(DesktopError::Full); }
        let id = self.compositor.create_app_window(
            awe_ayui::Rect { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
            app.app_type(), app.title()).map_err(|_| DesktopError::Window)?;
        self.windows[self.window_count] = Some((app, id.0));
        self.window_count += 1;
        Ok(id.0)
    }

    pub fn close(&mut self, app: DesktopApp) -> Result<(), DesktopError> {
        let index = self.windows.iter().position(|entry| entry.map(|(a, _)| a) == Some(app)).ok_or(DesktopError::Window)?;
        let (_, id) = self.windows[index].take().unwrap();
        self.compositor.destroy_window(awe_ayui::WindowId(id)).map_err(|_| DesktopError::Window)?;
        self.compact();
        Ok(())
    }

    pub fn dispatch_input(&mut self, event: awe_ayui::InputEvent) -> Result<(), DesktopError> {
        self.compositor.push_input(event).map_err(|_| DesktopError::Window)
    }

    pub fn tick(&mut self) { self.clock_ticks = self.clock_ticks.wrapping_add(1); }

    pub fn render(&self, buffer: &mut [u8]) -> Result<(), DesktopError> {
        let info = self.framebuffer.ok_or(DesktopError::InvalidFramebuffer)?;
        let required = info.required_bytes().ok_or(DesktopError::InvalidFramebuffer)? as usize;
        if buffer.len() < required { return Err(DesktopError::InvalidFramebuffer); }
        let mut fb = Framebuffer { width: info.width, height: info.height, stride: info.pitch / info.bytes_per_pixel as u32, buffer, gpu_accel: false };
        self.compositor.render_to_framebuffer(&mut fb);
        Ok(())
    }

    fn compact(&mut self) {
        let mut dst = 0;
        for src in 0..MAX_DESKTOP_APPS {
            if let Some(item) = self.windows[src] {
                if src != dst { self.windows[dst] = Some(item); self.windows[src] = None; }
                dst += 1;
            }
        }
        self.window_count = dst;
    }
}
impl Default for DesktopShell { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desktop_autostarts_core_windows_and_closes_them() {
        let mut shell = DesktopShell::new();
        shell.attach_framebuffer(FramebufferInfo { address: 0x100000, size: 800 * 600 * 4, width: 800, height: 600, pitch: 3200, bytes_per_pixel: 4 }).unwrap();
        shell.start().unwrap();
        assert_eq!(shell.window_count, 2);
        shell.close(DesktopApp::Terminal).unwrap();
        assert_eq!(shell.window_count, 1);
    }

    #[test]
    fn desktop_renders_only_into_valid_dynamic_framebuffer() {
        let mut shell = DesktopShell::new();
        assert_eq!(shell.render(&mut [0u8; 64]), Err(DesktopError::InvalidFramebuffer));
        shell.attach_framebuffer(FramebufferInfo { address: 0x200000, size: 320 * 240 * 4, width: 320, height: 240, pitch: 1280, bytes_per_pixel: 4 }).unwrap();
        let mut frame = [0u8; 320 * 240 * 4];
        shell.render(&mut frame).unwrap();
        assert!(frame.iter().any(|b| *b != 0));
    }
}