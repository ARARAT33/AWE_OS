#![no_std]

use awe_ayui::{AppType, Compositor, Rect};

pub struct TextEditorApp {
    pub window_width: u32,
    pub window_height: u32,
    pub document_buffer: [u8; 1024],
    pub document_len: usize,
}

impl TextEditorApp {
    pub fn new() -> Self {
        Self {
            window_width: 600,
            window_height: 400,
            document_buffer: [0u8; 1024],
            document_len: 0,
        }
    }

    pub fn launch(&self, compositor: &mut Compositor) {
        let bounds = Rect {
            x: 120,
            y: 80,
            width: self.window_width,
            height: self.window_height,
        };
        let _ = compositor.create_app_window(bounds, AppType::TextEditor, b"Text Editor");
    }
}

impl Default for TextEditorApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_editor_app() {
        let mut compositor = Compositor::new();
        let app = TextEditorApp::new();
        app.launch(&mut compositor);
        assert_eq!(compositor.window_count(), 1);
    }
}
