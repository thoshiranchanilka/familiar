pub mod normalize;

use tauri::{Manager, PhysicalPosition};

/// Margin between the overlay window and the screen's bottom-right corner, in logical pixels.
const CORNER_MARGIN: f64 = 16.0;

fn place_bottom_right(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.current_monitor()? {
        let scale = monitor.scale_factor();
        let area = monitor.size();
        let pos = monitor.position();
        let size = window.outer_size()?;
        let margin = (CORNER_MARGIN * scale) as i32;
        let x = pos.x + area.width as i32 - size.width as i32 - margin;
        let y = pos.y + area.height as i32 - size.height as i32 - margin;
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // No Dock icon and no app switcher entry: the mascot is an accessory overlay.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app.get_webview_window("main").expect("main window");
            place_bottom_right(&window)?;
            window.set_ignore_cursor_events(true)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
