mod commands;
mod models;
mod platform;
mod services;

use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use services::session_store::SessionStore;

#[tauri::command]
fn get_notch_geometry() -> platform::window::NotchGeometry {
    platform::window::get_notch_geometry()
}

#[tauri::command]
fn get_history_stats() -> services::history_stats::HistoryStats {
    services::history_stats::load()
}

#[tauri::command]
fn get_level_info() -> services::level_manager::LevelInfo {
    services::level_manager::get_info()
}

#[tauri::command]
fn preview_sound(name: String) {
    services::sound_service::play(&name);
}

#[tauri::command]
fn get_available_sounds() -> Vec<&'static str> {
    services::sound_service::available_sounds()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let (tx, rx) = broadcast::channel::<models::hook_event::HookEvent>(100);
    let store = Arc::new(Mutex::new(SessionStore::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::hooks::install_hooks,
            commands::hooks::uninstall_hooks,
            commands::hooks::is_hooks_installed,
            commands::settings::get_settings,
            commands::settings::update_settings,
            get_notch_geometry,
            get_history_stats,
            get_level_info,
            preview_sound,
            get_available_sounds,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // Position window at notch
            platform::window::position_window(&handle);

            // Start socket server
            services::socket_server::start(tx);

            // Start state machine
            services::state_machine::start(handle, store, rx);

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                services::socket_server::cleanup();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
