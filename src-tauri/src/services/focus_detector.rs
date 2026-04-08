/// Terminal and IDE bundle IDs — considered "productive" apps
const TERMINAL_IDS: &[&str] = &[
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "dev.warp.Warp-Stable",
    "dev.warp.Warp-Preview",
    "io.alacritty",
    "net.kovidgoyal.kitty",
    "com.github.wez.wezterm",
    "co.zeit.hyper",
    "com.mitchellh.ghostty",
    "com.raphaelamorim.rio",
    "org.tabby",
    "dev.commandline.waveterm",
    "org.contourterminal.Contour",
    // IDEs
    "com.microsoft.VSCode",
    "com.todesktop.230313mzl4w4u92",
    "com.exafunction.windsurf",
    "dev.zed.Zed",
    "com.jetbrains.intellij",
    "com.jetbrains.intellij.ce",
    "com.jetbrains.pycharm",
    "com.jetbrains.pycharm.ce",
];

/// Direct procrastination apps (always trigger idle)
const PROCRASTINATION_IDS: &[&str] = &[
    "com.spotify.client",
    "com.apple.TV",
    "com.tinyspeck.slackmacgap",
    "com.hnc.Discord",
    "com.facebook.archon",
    "com.telegram.desktop",
    "com.whatsapp.WhatsApp",
];

/// Check if the current frontmost app is a terminal/IDE
pub fn is_terminal_focused() -> bool {
    if let Some(bundle_id) = get_frontmost_bundle_id() {
        return TERMINAL_IDS.contains(&bundle_id.as_str());
    }
    false
}

/// Detect if user is on a procrastination app
pub fn is_procrastinating() -> bool {
    let bundle_id = match get_frontmost_bundle_id() {
        Some(id) => id,
        None => return false,
    };

    // Direct procrastination app
    if PROCRASTINATION_IDS.contains(&bundle_id.as_str()) {
        return true;
    }

    // Terminal/IDE = definitely not procrastinating
    if TERMINAL_IDS.contains(&bundle_id.as_str()) {
        return false;
    }

    false
}

#[cfg(target_os = "macos")]
fn get_frontmost_bundle_id() -> Option<String> {
    use cocoa::base::{id, nil};
    use objc::{msg_send, sel, sel_impl, class};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let app: id = msg_send![workspace, frontmostApplication];
        if app == nil {
            return None;
        }
        let bundle_id: id = msg_send![app, bundleIdentifier];
        if bundle_id == nil {
            return None;
        }
        let utf8: *const i8 = msg_send![bundle_id, UTF8String];
        if utf8.is_null() {
            return None;
        }
        Some(std::ffi::CStr::from_ptr(utf8).to_string_lossy().into_owned())
    }
}

#[cfg(not(target_os = "macos"))]
fn get_frontmost_bundle_id() -> Option<String> {
    None
}
