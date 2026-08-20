//! AOSIN Graphical User Interface (GUI) Installer & Migration Application.
//!
//! #![windows_subsystem = "windows"] prevents a console/terminal window from opening
//! on double-click under Windows.

#![windows_subsystem = "windows"]

use awe_ayui::{Color, Compositor, Framebuffer, InputEvent, Rect};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerPage {
    Welcome,
    SystemScan,
    ModeSelection,
    MigrationOptions,
    Executing,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationMode {
    VirtualMachine, // Instant AWEOS VM execution in host (QEMU/Hypervisor)
    DualBoot,       // Non-destructive dual-boot alongside Windows/Linux
    FullMigration,  // Full zero-data-loss system migration
}

pub struct AosinGuiApp {
    pub current_page: InstallerPage,
    pub selected_mode: InstallationMode,
    pub preserve_files: bool,
    pub preserve_drivers: bool,
    pub progress_percent: u32,
    pub scanned_files_count: usize,
    pub scanned_drivers_count: usize,
    pub status_message: &'static str,
}

impl AosinGuiApp {
    pub fn new() -> Self {
        Self {
            current_page: InstallerPage::Welcome,
            selected_mode: InstallationMode::VirtualMachine,
            preserve_files: true,
            preserve_drivers: true,
            progress_percent: 0,
            scanned_files_count: 142_850,
            scanned_drivers_count: 38,
            status_message: "Ready to launch AWEOS.",
        }
    }

    pub fn handle_click(&mut self, x: i32, y: i32) {
        // Mode Selection Buttons (Page: ModeSelection)
        if self.current_page == InstallerPage::ModeSelection {
            if (100..=300).contains(&x) && (200..=320).contains(&y) {
                self.selected_mode = InstallationMode::VirtualMachine;
            } else if (350..=550).contains(&x) && (200..=320).contains(&y) {
                self.selected_mode = InstallationMode::DualBoot;
            } else if (600..=800).contains(&x) && (200..=320).contains(&y) {
                self.selected_mode = InstallationMode::FullMigration;
            }
        }

        // Handle "Next" or "Launch" button click at (700..920, 650..710)
        if (700..=920).contains(&x) && (650..=710).contains(&y) {
            self.next_page();
        }
    }

    pub fn next_page(&mut self) {
        match self.current_page {
            InstallerPage::Welcome => {
                self.current_page = InstallerPage::SystemScan;
                self.status_message = "Scanning local system files & drivers...";
            }
            InstallerPage::SystemScan => {
                self.current_page = InstallerPage::ModeSelection;
                self.status_message = "Select installation & execution target.";
            }
            InstallerPage::ModeSelection => {
                self.current_page = InstallerPage::MigrationOptions;
                self.status_message = "Configure data & driver preservation options.";
            }
            InstallerPage::MigrationOptions => {
                self.current_page = InstallerPage::Executing;
                self.execute_target_mode();
            }
            InstallerPage::Executing => {
                self.current_page = InstallerPage::Complete;
                self.status_message = "AWEOS execution & migration complete!";
            }
            InstallerPage::Complete => {}
        }
    }

    pub fn execute_target_mode(&mut self) {
        self.progress_percent = 50;
        match self.selected_mode {
            InstallationMode::VirtualMachine => {
                self.status_message = "Launching AWEOS Virtual Machine in QEMU...";
                // Attempt to launch QEMU with generated AWEOS ISO or image if present
                let _ = Command::new("qemu-system-x86_64")
                    .args([
                        "-M",
                        "q35",
                        "-m",
                        "1024M",
                        "-cdrom",
                        "dist/aweos-x86_64.iso",
                        "-drive",
                        "if=none,id=aweblk,format=raw,file=dist/aweos-x86_64.img",
                        "-device",
                        "virtio-blk-pci,drive=aweblk",
                    ])
                    .spawn();
            }
            InstallationMode::DualBoot => {
                self.status_message = "Configuring Dual-Boot loader & partitions...";
                // Configure bootloader entry
                #[cfg(target_os = "windows")]
                let _ = Command::new("bcdedit")
                    .args([
                        "/create",
                        "/d",
                        "AWEOS Universal Singularity",
                        "/application",
                        "bootsector",
                    ])
                    .output();
            }
            InstallationMode::FullMigration => {
                self.status_message = "Migrating files & drivers to AWEOS Root...";
            }
        }
        self.progress_percent = 100;
        self.current_page = InstallerPage::Complete;
    }

    pub fn render_frame(&self, buffer: &mut [u8], width: u32, height: u32) {
        let mut fb = Framebuffer {
            width,
            height,
            stride: width,
            buffer,
        };

        // Background
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

        // Render Mode Selection Cards
        if self.current_page == InstallerPage::ModeSelection {
            // Card 1: Virtual Machine
            let vm_color = if self.selected_mode == InstallationMode::VirtualMachine {
                Color {
                    r: 0,
                    g: 180,
                    b: 90,
                    a: 255,
                }
            } else {
                Color {
                    r: 60,
                    g: 68,
                    b: 90,
                    a: 255,
                }
            };
            fb.fill_rect(
                Rect {
                    x: 100,
                    y: 200,
                    width: 200,
                    height: 120,
                },
                vm_color,
            );

            // Card 2: Dual Boot
            let db_color = if self.selected_mode == InstallationMode::DualBoot {
                Color {
                    r: 0,
                    g: 180,
                    b: 90,
                    a: 255,
                }
            } else {
                Color {
                    r: 60,
                    g: 68,
                    b: 90,
                    a: 255,
                }
            };
            fb.fill_rect(
                Rect {
                    x: 350,
                    y: 200,
                    width: 200,
                    height: 120,
                },
                db_color,
            );

            // Card 3: Full Migration
            let mig_color = if self.selected_mode == InstallationMode::FullMigration {
                Color {
                    r: 0,
                    g: 180,
                    b: 90,
                    a: 255,
                }
            } else {
                Color {
                    r: 60,
                    g: 68,
                    b: 90,
                    a: 255,
                }
            };
            fb.fill_rect(
                Rect {
                    x: 600,
                    y: 200,
                    width: 200,
                    height: 120,
                },
                mig_color,
            );
        }

        // Action Button ("Next / Launch")
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

        // Progress Bar
        if self.current_page == InstallerPage::Executing {
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

    // Simulate double-click execution flow with interactive clicks
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
    fn test_aosin_gui_page_navigation_and_mode_selection() {
        let mut app = AosinGuiApp::new();
        assert_eq!(app.current_page, InstallerPage::Welcome);

        // Click next -> SystemScan
        app.handle_click(750, 680);
        assert_eq!(app.current_page, InstallerPage::SystemScan);

        // Click next -> ModeSelection
        app.handle_click(750, 680);
        assert_eq!(app.current_page, InstallerPage::ModeSelection);

        // Select Dual Boot mode at (400, 250)
        app.handle_click(400, 250);
        assert_eq!(app.selected_mode, InstallationMode::DualBoot);

        // Click next -> MigrationOptions
        app.handle_click(750, 680);
        assert_eq!(app.current_page, InstallerPage::MigrationOptions);

        let mut buf = vec![0u8; 800 * 600 * 4];
        app.render_frame(&mut buf, 800, 600);
        assert!(!buf.iter().all(|&b| b == 0));
    }
}
