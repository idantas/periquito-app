use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Debug, Clone, Serialize)]
pub struct NotchGeometry {
    pub notch_width: f64,
    pub notch_height: f64,
    pub screen_width: f64,
    pub screen_height: f64,
}

impl Default for NotchGeometry {
    fn default() -> Self {
        Self {
            notch_width: 224.0,
            notch_height: 38.0,
            screen_width: 1440.0,
            screen_height: 900.0,
        }
    }
}

pub fn get_notch_geometry() -> NotchGeometry {
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSScreen;
        use cocoa::base::nil;

        unsafe {
            let main_screen = NSScreen::mainScreen(nil);
            if main_screen.is_null() {
                return NotchGeometry::default();
            }

            #[allow(deprecated)]
            let frame = NSScreen::frame(main_screen);

            // Use safe area top for notch height detection
            // For MacBooks with notch, the menu bar height is typically 38px
            // We use a reasonable default that works for all MacBook models with notch
            NotchGeometry {
                notch_width: 224.0,
                notch_height: 38.0,
                #[allow(deprecated)]
                screen_width: frame.size.width,
                #[allow(deprecated)]
                screen_height: frame.size.height,
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    NotchGeometry::default()
}

pub fn position_window(app: &AppHandle) {
    let geometry = get_notch_geometry();

    if let Some(window) = app.get_webview_window("main") {
        let panel_width = 500.0_f64;
        let panel_height = 500.0_f64;

        // Center horizontally on screen
        let x = (geometry.screen_width - panel_width) / 2.0;
        let y = 0.0;

        let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
        let _ = window.set_size(tauri::PhysicalSize::new(panel_width as u32, panel_height as u32));
        let _ = window.set_always_on_top(true);

        #[cfg(target_os = "macos")]
        set_window_level(&window);

        log::info!(
            "Window positioned: {}x{} at ({}, {})",
            panel_width, panel_height, x, y
        );
    }
}

#[cfg(target_os = "macos")]
fn set_window_level(window: &WebviewWindow) {
    use cocoa::appkit::NSWindow;

    if let Ok(ns_window) = window.ns_window() {
        unsafe {
            let ns_win = ns_window as cocoa::base::id;
            // NSMainMenuWindowLevel + 3 = 24 + 3 = 27
            #[allow(deprecated)]
            ns_win.setLevel_(27);
            // Appear on all spaces
            #[allow(deprecated)]
            ns_win.setCollectionBehavior_(
                cocoa::appkit::NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                    | cocoa::appkit::NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary,
            );
        }
    }
}
