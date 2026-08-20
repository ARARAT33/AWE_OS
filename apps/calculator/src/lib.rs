//! Native AWEOS Calculator Application.

use awe_ayui::{Color, Framebuffer, Rect};

pub struct AweCalculatorApp {
    pub display_value: f64,
    pub accumulator: f64,
}

impl AweCalculatorApp {
    pub fn new() -> Self {
        Self {
            display_value: 0.0,
            accumulator: 0.0,
        }
    }

    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
        };

        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            Color {
                r: 16,
                g: 18,
                b: 26,
                a: 255,
            },
        );
    }
}

impl Default for AweCalculatorApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_app() {
        let app = AweCalculatorApp::new();
        assert_eq!(app.display_value, 0.0);
        let mut buf = vec![0u8; 400 * 500 * 4];
        app.render(&mut buf, 400, 500);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
