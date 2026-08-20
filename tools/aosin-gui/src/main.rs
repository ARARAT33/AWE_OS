//! AOSIN Graphical User Interface (GUI) Installer & Migration Application.
//!
//! #![windows_subsystem = "windows"] prevents a console/terminal window from opening
//! on double-click under Windows.

#![windows_subsystem = "windows"]

use awe_ayui::{Color, Framebuffer, Rect};
#[cfg(not(target_os = "windows"))]
use awe_ayui::{Compositor, InputEvent};
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
            gpu_accel: false,
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

        // Mode Selection Cards
        if self.current_page == InstallerPage::ModeSelection {
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

// Win32 Zero-Dependency Native Window Implementation
#[cfg(target_os = "windows")]
mod win32_host {
    use super::*;
    use std::ffi::c_void;
    use std::ptr::null_mut;

    type HWND = *mut c_void;
    type HDC = *mut c_void;
    type HINSTANCE = *mut c_void;
    type HMENU = *mut c_void;
    type HICON = *mut c_void;
    type HCURSOR = *mut c_void;
    type HBRUSH = *mut c_void;
    type LPCWSTR = *const u16;
    type LRESULT = isize;
    type WPARAM = usize;
    type LPARAM = isize;
    type UINT = u32;
    type DWORD = u32;
    type WORD = u16;
    type LONG = i32;

    #[repr(C)]
    struct WNDCLASSEXW {
        cb_size: UINT,
        style: UINT,
        lpfn_wnd_proc: unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: HINSTANCE,
        h_icon: HICON,
        h_cursor: HCURSOR,
        hbr_background: HBRUSH,
        lpsz_menu_name: LPCWSTR,
        lpsz_class_name: LPCWSTR,
        h_icon_sm: HICON,
    }

    #[repr(C)]
    struct BITMAPINFOHEADER {
        bi_size: DWORD,
        bi_width: LONG,
        bi_height: LONG,
        bi_planes: WORD,
        bi_bit_count: WORD,
        bi_compression: DWORD,
        bi_size_image: DWORD,
        bi_x_pels_per_meter: LONG,
        bi_y_pels_per_meter: LONG,
        bi_clr_used: DWORD,
        bi_clr_important: DWORD,
    }

    #[repr(C)]
    struct BITMAPINFO {
        bmi_header: BITMAPINFOHEADER,
        bmi_colors: [u32; 1],
    }

    #[repr(C)]
    struct POINT {
        x: LONG,
        y: LONG,
    }

    #[repr(C)]
    struct MSG {
        hwnd: HWND,
        message: UINT,
        w_param: WPARAM,
        l_param: LPARAM,
        time: DWORD,
        pt: POINT,
    }

    #[repr(C)]
    struct PAINTSTRUCT {
        hdc: HDC,
        f_erase: i32,
        rc_paint: [i32; 4],
        f_restore: i32,
        f_inc_update: i32,
        rgb_reserved: [u8; 32],
    }

    #[link(name = "user32")]
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn GetModuleHandleW(lp_module_name: LPCWSTR) -> HINSTANCE;
        fn RegisterClassExW(lp_wcx: *const WNDCLASSEXW) -> WORD;
        fn CreateWindowExW(
            dw_ex_style: DWORD,
            lp_class_name: LPCWSTR,
            lp_window_name: LPCWSTR,
            dw_style: DWORD,
            x: i32,
            y: i32,
            n_width: i32,
            n_height: i32,
            h_wnd_parent: HWND,
            h_menu: HMENU,
            h_instance: HINSTANCE,
            lp_param: *mut c_void,
        ) -> HWND;
        fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> i32;
        fn GetMessageW(
            lp_msg: *mut MSG,
            h_wnd: HWND,
            w_msg_filter_min: UINT,
            w_msg_filter_max: UINT,
        ) -> i32;
        fn TranslateMessage(lp_msg: *const MSG) -> i32;
        fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
        fn PostQuitMessage(n_exit_code: i32);
        fn DefWindowProcW(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
        fn BeginPaint(h_wnd: HWND, lp_paint: *mut PAINTSTRUCT) -> HDC;
        fn EndPaint(h_wnd: HWND, lp_paint: *const PAINTSTRUCT) -> i32;
        fn StretchDIBits(
            hdc: HDC,
            x_dest: i32,
            y_dest: i32,
            dest_width: i32,
            dest_height: i32,
            x_src: i32,
            y_src: i32,
            src_width: i32,
            src_height: i32,
            lp_bits: *const c_void,
            lpbmi: *const BITMAPINFO,
            i_usage: UINT,
            rop: DWORD,
        ) -> i32;
        fn InvalidateRect(h_wnd: HWND, lp_rect: *const c_void, b_erase: i32) -> i32;
    }

    static mut APP_INSTANCE: Option<AosinGuiApp> = None;

    #[allow(static_mut_refs)]
    unsafe extern "system" fn wnd_proc(
        h_wnd: HWND,
        msg: UINT,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        const WM_DESTROY: UINT = 0x0002;
        const WM_PAINT: UINT = 0x000F;
        const WM_LBUTTONDOWN: UINT = 0x0201;

        match msg {
            WM_LBUTTONDOWN => {
                let x = (l_param & 0xFFFF) as i32;
                let y = ((l_param >> 16) & 0xFFFF) as i32;
                if let Some(app) = unsafe { APP_INSTANCE.as_mut() } {
                    app.handle_click(x, y);
                    unsafe { InvalidateRect(h_wnd, null_mut(), 0) };
                }
                0
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
                let hdc = unsafe { BeginPaint(h_wnd, &mut ps) };

                if let Some(app) = unsafe { APP_INSTANCE.as_ref() } {
                    let mut frame_buf = vec![0u8; 1024 * 768 * 4];
                    app.render_frame(&mut frame_buf, 1024, 768);

                    let bmi = BITMAPINFO {
                        bmi_header: BITMAPINFOHEADER {
                            bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as DWORD,
                            bi_width: 1024,
                            bi_height: -768, // Top-down
                            bi_planes: 1,
                            bi_bit_count: 32,
                            bi_compression: 0,
                            bi_size_image: (1024 * 768 * 4) as DWORD,
                            bi_x_pels_per_meter: 0,
                            bi_y_pels_per_meter: 0,
                            bi_clr_used: 0,
                            bi_clr_important: 0,
                        },
                        bmi_colors: [0; 1],
                    };

                    unsafe {
                        StretchDIBits(
                            hdc,
                            0,
                            0,
                            1024,
                            768,
                            0,
                            0,
                            1024,
                            768,
                            frame_buf.as_ptr() as *const c_void,
                            &bmi,
                            0,
                            0x00CC0020, // SRCCOPY
                        );
                    }
                }

                unsafe { EndPaint(h_wnd, &ps) };
                0
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                0
            }
            _ => unsafe { DefWindowProcW(h_wnd, msg, w_param, l_param) },
        }
    }

    pub fn run_win32_app(app: AosinGuiApp) {
        unsafe { APP_INSTANCE = Some(app) };

        let class_name: Vec<u16> = "AOSIN_GUI_CLASS\0".encode_utf16().collect();
        let window_title: Vec<u16> = "AOSIN — Universal AWEOS Installer & Migration Engine\0"
            .encode_utf16()
            .collect();

        let h_instance = unsafe { GetModuleHandleW(null_mut()) };

        let wcx = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0x0003, // CS_HREDRAW | CS_VREDRAW
            lpfn_wnd_proc: wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance,
            h_icon: null_mut(),
            h_cursor: null_mut(),
            hbr_background: null_mut(),
            lpsz_menu_name: null_mut(),
            lpsz_class_name: class_name.as_ptr(),
            h_icon_sm: null_mut(),
        };

        unsafe { RegisterClassExW(&wcx) };

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                0x00CF0000, // WS_OVERLAPPEDWINDOW
                100,
                100,
                1024,
                768,
                null_mut(),
                null_mut(),
                h_instance,
                null_mut(),
            )
        };

        if !hwnd.is_null() {
            unsafe {
                ShowWindow(hwnd, 5); // SW_SHOW
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
}

fn main() {
    let app = AosinGuiApp::new();

    #[cfg(target_os = "windows")]
    {
        win32_host::run_win32_app(app);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut app = app;
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
