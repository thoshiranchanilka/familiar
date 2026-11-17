pub mod normalize;
pub mod server;
pub mod token;
pub mod installer;
pub mod desktop_monitor;

use tauri::{Emitter, Manager, PhysicalPosition};

/// Last event-bridge failure, readable by the overlay even if it loads after the failure.
struct BridgeError(std::sync::Mutex<Option<String>>);

fn report_bridge_error(app: &tauri::AppHandle, msg: String) {
    *app.state::<BridgeError>().0.lock().unwrap() = Some(msg.clone());
    let _ = app.emit("mascot-server-error", msg);
}

#[tauri::command]
fn bridge_error(state: tauri::State<BridgeError>) -> Option<String> {
    state.0.lock().unwrap().clone()
}

fn home(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().home_dir().map_err(|e| e.to_string())
}

fn install_error_text(e: installer::InstallError) -> String {
    match e {
        installer::InstallError::InvalidSettings(m) => format!(
            "Your Claude Code settings file couldn't be read, so nothing was changed. {m}"
        ),
        installer::InstallError::Io(m) => format!("Couldn't update Claude Code settings: {m}"),
    }
}

/// Connects Claude Code: adds the mascot's hooks and status line.
#[tauri::command]
fn mascot_install(app: tauri::AppHandle) -> Result<(), String> {
    let home = home(&app)?;
    let token = token::load_or_create_token(&home).map_err(|e| e.to_string())?;
    installer::install(&home, &token, server::DEFAULT_PORT).map(|_| ()).map_err(install_error_text)
}

/// Disconnects Claude Code and restores the previous settings.
#[tauri::command]
fn mascot_uninstall(app: tauri::AppHandle) -> Result<(), String> {
    installer::uninstall(&home(&app)?).map_err(install_error_text)
}

#[tauri::command]
fn mascot_status(app: tauri::AppHandle) -> Result<bool, String> {
    Ok(installer::is_installed(&home(&app)?))
}

/// Starts the event server and forwards every event to the overlay as `mascot-event`.
/// A port clash is reported to the overlay as `mascot-server-error`.
fn start_event_bridge(app: &tauri::AppHandle) {
    let home = app.path().home_dir().expect("home dir");
    let token = match token::load_or_create_token(&home) {
        Ok(t) => t,
        Err(e) => {
            report_bridge_error(app, format!("Could not create the token file: {e}"));
            return;
        }
    };
    let (tx, rx) = std::sync::mpsc::channel();
    desktop_monitor::spawn(tx.clone());
    match server::start(token, server::DEFAULT_PORT, tx) {
        Ok(handle) => {
            app.manage(handle); // keep it alive for the app's lifetime
            let app = app.clone();
            std::thread::spawn(move || {
                for ev in rx {
                    let _ = app.emit("mascot-event", ev);
                }
            });
        }
        Err(e) => {
            let msg = match e {
                server::ServerError::PortInUse => format!(
                    "Mascot already running, or port {} is busy.",
                    server::DEFAULT_PORT
                ),
                server::ServerError::Io(m) => m,
            };
            report_bridge_error(app, msg);
        }
    }
}

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
        .manage(BridgeError(std::sync::Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![bridge_error, mascot_install, mascot_uninstall, mascot_status])
        .setup(|app| {
            // No Dock icon and no app switcher entry: the mascot is an accessory overlay.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app.get_webview_window("main").expect("main window");
            place_bottom_right(&window)?;
            window.set_ignore_cursor_events(true)?;
            start_event_bridge(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
