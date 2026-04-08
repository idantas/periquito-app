use crate::services::hook_installer;

const HOOK_SCRIPT: &str = include_str!("../../resources/periquito-hook.sh");

#[tauri::command]
pub fn install_hooks() -> Result<bool, String> {
    hook_installer::install(HOOK_SCRIPT)
}

#[tauri::command]
pub fn uninstall_hooks() {
    hook_installer::uninstall();
}

#[tauri::command]
pub fn is_hooks_installed() -> bool {
    hook_installer::is_installed()
}
