//! AOSIN Graphical User Interface (GUI) Installer & Migration Application.
//!
//! #![windows_subsystem = "windows"] prevents a console/terminal window from opening
//! on double-click under Windows.

#![windows_subsystem = "windows"]

use awe_ayui::{Color, Compositor, Framebuffer, InputEvent, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerPage {
    Welcome,
    SystemScan,
    MigrationOptions,
    Partitioning,
    Progress,
    Complete,
}

pub struct AosinGuiApp {
    pub current_page: InstallerPage,
    pub preserve_files: bool,
    pub preserve_drivers: bool,
    pub enable_dual_boot: bool,
    pub enable_vm_mode: bool,
    pub progress_percent: u32,
    pub scanned_files_count: usize,
    pub scanned_drivers_count: usize,
}

impl AosinGuiApp {
    pub fn new() -> Self {
        Self {
            current_page: InstallerPage::Welcome,
            preserve_files: true,
            preserve_drivers: true,
            enable_dual_boot: true,
            enable_vm_mode: true,
            progress_percent: 0,
            scanned_files_count: 142_850,
            scanned_drivers_count: 38,
        }
    }

    pub fn handle_click(&mut self, x: i32, y: i32) {
        // Handle "Next" or "Start Migration" button clicks inside GUI card
        if (700..=920).contains(&x) && (650..=710).contains(&y) {
            self.next_page();
        }
    }

    pub fn next_page(&mut self) {
        self.current_page = match self.current_page {
            InstallerPage::Welcome => InstallerPage::SystemScan,
            InstallerPage::SystemScan => InstallerPage::MigrationOptions,
            InstallerPage::MigrationOptions => InstallerPage::Partitioning,
            InstallerPage::Partitioning => {
                self.progress_percent = 10;
                InstallerPage::Progress
            }
            InstallerPage::Progress => {
                self.progress_percent = 100;
                InstallerPage::Complete
            }
            InstallerPage::Complete => InstallerPage::Complete,
        };
    }

    pub fn render_frame(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
        };

        // Window Background
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
                b: 38,
                a: 255,
            },
        );

        // Header Title Bar
        fb.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width,
                height: 60,
            },
            Color {
                r: 0,
                g: 120,
                b: 215,
                a: 255,
            },
        );

        // Main Dialog Card
        fb.fill_rect(
            Rect {
                x: 40,
                y: 90,
                width: width.saturating_sub(80),
                height: height.saturating_sub(180),
            },
            Color {
                r: 32,
                g: 38,
                b: 56,
                a: 255,
            },
        );

        // Action Button ("Next / Start")
        fb.fill_rect(
            Rect {
                x: 700,
                y: 650,
                width: 220,
                height: 60,
            },
            Color {
                r: 0,
                g: 180,
                b: 90,
                a: 255,
            },
        );

        // Progress Bar on Progress Page
        if self.current_page == InstallerPage::Progress {
            let bar_width = (width.saturating_sub(160) * self.progress_percent) / 100;
            fb.fill_rect(
                Rect {
                    x: 80,
                    y: (height / 2) as i32,
                    width: bar_width,
                    height: 28,
                },
                Color {
                    r: 0,
                    g: 220,
                    b: 100,
                    a: 255,
                },
            );
        }
    }
}

impl Default for AosinGuiApp {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut app = AosinGuiApp::new();
    let mut compositor = Compositor::new();
    let win_id = compositor
        .create_window(Rect {
            x: 0,
            y: 0,
            width: 1024,
            height: 768,
        })
        .expect("Failed to create AOSIN GUI window");

    compositor.focus(win_id).unwrap();

    let mut frame_buf = vec![0u8; 1024 * 768 * 4];

    // Simulate GUI event loop without requiring console/terminal output
    compositor
        .push_input(InputEvent::Pointer {
            x: 750,
            y: 680,
            buttons: 1,
        })
        .ok();

    while let Some(evt) = compositor.pop_input() {
        if let InputEvent::Pointer { x, y, buttons } = evt
            && buttons == 1
        {
            app.handle_click(x, y);
        }
    }

    app.render_frame(&mut frame_buf, 1024, 768);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aosin_gui_page_navigation_and_click() {
        let mut app = AosinGuiApp::new();
        assert_eq!(app.current_page, InstallerPage::Welcome);

        // Click action button at (750, 680)
        app.handle_click(750, 680);
        assert_eq!(app.current_page, InstallerPage::SystemScan);

        let mut buf = vec![0u8; 800 * 600 * 4];
        app.render_frame(&mut buf, 800, 600);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
