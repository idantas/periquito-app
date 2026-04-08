use crate::models::app_settings::AppSettings;

#[tauri::command]
pub fn get_settings() -> AppSettings {
    AppSettings::load()
}

#[tauri::command]
pub fn update_settings(settings: AppSettings) -> Result<(), String> {
    settings.save()
}
