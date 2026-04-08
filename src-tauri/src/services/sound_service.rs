use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::app_settings::AppSettings;

static LAST_PLAY: AtomicU64 = AtomicU64::new(0);
const COOLDOWN_MS: u64 = 2000;

/// Play a system sound by name (e.g. "Glass", "Pop", "Purr").
/// Respects mute setting and 2s cooldown between plays.
pub fn play_for_tip(tip_type: &str) {
    let settings = AppSettings::load();
    if settings.is_muted {
        return;
    }

    let sound_name = match tip_type {
        "good" => "Glass",
        "correction" => "Pop",
        _ => return,
    };

    play(sound_name);
}

/// Play a specific sound by name (for preview in settings).
pub fn play(name: &str) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let last = LAST_PLAY.load(Ordering::Relaxed);
    if now - last < COOLDOWN_MS {
        return;
    }
    LAST_PLAY.store(now, Ordering::Relaxed);

    let sound_name = name.to_string();
    std::thread::spawn(move || {
        play_nssound(&sound_name);
    });
}

#[cfg(target_os = "macos")]
fn play_nssound(name: &str) {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{msg_send, sel, sel_impl, class};

    unsafe {
        let ns_name = NSString::alloc(nil).init_str(name);
        let sound: id = msg_send![class!(NSSound), soundNamed: ns_name];
        if sound != nil {
            let _: () = msg_send![sound, play];
        } else {
            log::warn!("Sound not found: {}", name);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn play_nssound(name: &str) {
    log::info!("Sound playback not supported on this platform: {}", name);
}

/// Available macOS system sounds.
pub fn available_sounds() -> Vec<&'static str> {
    vec![
        "Basso", "Blow", "Bottle", "Frog", "Funk",
        "Glass", "Hero", "Morse", "Ping", "Pop",
        "Purr", "Sosumi", "Submarine", "Tink",
    ]
}
