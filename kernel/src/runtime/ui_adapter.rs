#![no_std]

use awe_ayui::{AppType, Compositor, Framebuffer, InputEvent as AyuiInputEvent, Rect as AyuiRect};
use super::{FramebufferInfo, InputEvent, RuntimeRect};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRuntimeError {
    InvalidFramebuffer,
    RenderFailed,
    WindowFailed,
}

pub struct AyuiRuntime {
    pub compositor: Compositor,
    framebuffer: Option<FramebufferInfo>,
}

impl AyuiRuntime {
    pub const fn new() -> Self { Self { compositor: Compositor::new(), framebuffer: None } }

    pub fn attach_framebuffer(&mut self, info: FramebufferInfo) -> Result<(), UiRuntimeError> {
        if !info.validate() { return Err(UiRuntimeError::InvalidFramebuffer); }
        self.framebuffer = Some(info);
        Ok(())
    }

    pub fn create_window(&mut self, rect: RuntimeRect, app: AppType, title: &[u8]) -> Result<u16, UiRuntimeError> {
        self.compositor
            .create_app_window(AyuiRect { x: rect.x, y: rect.y, width: rect.width, height: rect.height }, app, title)
            .map(|id| id.0)
            .map_err(|_| UiRuntimeError::WindowFailed)
    }

    pub fn destroy_window(&mut self, id: u16) -> Result<(), UiRuntimeError> {
        self.compositor.destroy_window(awe_ayui::WindowId(id)).map_err(|_| UiRuntimeError::WindowFailed)
    }

    pub fn route_input(&mut self, event: InputEvent) -> Result<(), UiRuntimeError> {
        let ayui = match event {
            InputEvent::Key { code, pressed } => AyuiInputEvent::Key { code, pressed },
            InputEvent::Pointer { x, y, buttons } => AyuiInputEvent::Pointer { x, y, buttons },
        };
        self.compositor.push_input(ayui).map_err(|_| UiRuntimeError::WindowFailed)
    }

    pub fn render(&mut self, buffer: &mut [u8]) -> Result<(), UiRuntimeError> {
        let info = self.framebuffer.ok_or(UiRuntimeError::InvalidFramebuffer)?;
        let required = info.required_bytes().ok_or(UiRuntimeError::InvalidFramebuffer)? as usize;
        if buffer.len() < required { return Err(UiRuntimeError::InvalidFramebuffer); }
        let mut fb = Framebuffer {
            width: info.width,
            height: info.height,
            stride: info.pitch / info.bytes_per_pixel as u32,
            buffer,
            gpu_accel: false,
        };
        self.compositor.render_to_framebuffer(&mut fb);
        Ok(())
    }

    pub const fn framebuffer(&self) -> Option<FramebufferInfo> { self.framebuffer }
}
impl Default for AyuiRuntime { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dynamic_framebuffer_and_input_path_work() {
        let mut ui = AyuiRuntime::new();
        let info = FramebufferInfo { address: 0x100000, size: 640 * 480 * 4, width: 640, height: 480, pitch: 640 * 4, bytes_per_pixel: 4 };
        ui.attach_framebuffer(info).unwrap();
        let id = ui.create_window(RuntimeRect { x: 20, y: 20, width: 240, height: 160 }, AppType::Terminal, b"Terminal").unwrap();
        ui.route_input(InputEvent::Pointer { x: 30, y: 30, buttons: 1 }).unwrap();
        ui.route_input(InputEvent::Key { code: 0x1c, pressed: true }).unwrap();
        let mut frame = [0u8; 640 * 480 * 4];
        ui.render(&mut frame).unwrap();
        assert!(frame.iter().any(|b| *b != 0));
        ui.destroy_window(id).unwrap();
    }
}