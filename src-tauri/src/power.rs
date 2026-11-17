//! macOS Low Power Mode: while it is on, the mascot shows still frames only.

use tauri::{AppHandle, Emitter};

const CHECK_EVERY: std::time::Duration = std::time::Duration::from_secs(30);

#[cfg(target_os = "macos")]
pub fn low_power_mode() -> bool {
    objc2::rc::autoreleasepool(|_| objc2_foundation::NSProcessInfo::processInfo().isLowPowerModeEnabled())
}

#[cfg(not(target_os = "macos"))]
pub fn low_power_mode() -> bool {
    false
}

/// Checks every 30 s and emits `mascot-low-power` (bool) when the mode changes.
pub fn spawn_watch(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last = low_power_mode();
        loop {
            std::thread::sleep(CHECK_EVERY);
            let now = low_power_mode();
            if now != last {
                let _ = app.emit("mascot-low-power", now);
                last = now;
            }
        }
    });
}

#[tauri::command]
pub fn low_power() -> bool {
    low_power_mode()
}
