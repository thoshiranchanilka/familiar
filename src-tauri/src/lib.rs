pub mod normalize;
pub mod server;
pub mod token;
pub mod installer;
pub mod desktop_monitor;
pub mod settings;
pub mod layout;
pub mod power;

use layout::Rect;
use settings::{Corner, Settings};
use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Wry};

/// Last event-bridge failure, readable by the overlay even if it loads after the failure.
struct BridgeError(Mutex<Option<String>>);

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
    let result = installer::install(&home, &token, server::DEFAULT_PORT).map(|_| ()).map_err(install_error_text);
    refresh_tray(&app);
    result
}

/// Disconnects Claude Code and restores the previous settings.
#[tauri::command]
fn mascot_uninstall(app: tauri::AppHandle) -> Result<(), String> {
    let result = installer::uninstall(&home(&app)?).map_err(install_error_text);
    refresh_tray(&app);
    result
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
    // Earlier versions used HTTP hooks, which print errors in Claude Code whenever the
    // app is closed and receive full tool payloads. Switch them to the script hook.
    if installer::needs_migration(&home) {
        if let Err(e) = installer::install(&home, &token, server::DEFAULT_PORT) {
            report_bridge_error(app, install_error_text(e));
        }
    }
    let (tx, rx) = std::sync::mpsc::channel();
    match server::start(token, server::DEFAULT_PORT, tx.clone()) {
        Ok(handle) => {
            // Only the instance that owns the port watches Claude Desktop.
            desktop_monitor::spawn(tx);
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

/// Margin between the overlay window and the screen edges, in logical pixels.
const CORNER_MARGIN: f64 = 16.0;
/// Hover checks 4×/s; while the mascot is hidden, only a cheap settings check once a second.
const HOVER_POLL: std::time::Duration = std::time::Duration::from_millis(250);
const HIDDEN_POLL: std::time::Duration = std::time::Duration::from_millis(1000);
/// Re-read the window geometry at least this often (it can move to another display).
const GEOMETRY_REFRESH: std::time::Duration = std::time::Duration::from_secs(10);
/// Distance from the window's corner to the mascot; matches `#stage` in overlay.ts.
const MASCOT_INSET: f64 = 12.0;
const SIZES: [(&str, &str, u32); 3] = [("size-s", "Small", 96), ("size-m", "Medium", 128), ("size-l", "Large", 176)];
const CORNERS: [(&str, &str, Corner); 4] = [
    ("corner-br", "Bottom right", Corner::Br),
    ("corner-bl", "Bottom left", Corner::Bl),
    ("corner-tr", "Top right", Corner::Tr),
    ("corner-tl", "Top left", Corner::Tl),
];

struct AppSettings(Mutex<Settings>);

fn current_settings(app: &AppHandle) -> Settings {
    app.state::<AppSettings>().0.lock().unwrap().clone()
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    current_settings(&app)
}

/// Moves the window into the chosen corner of the screen's usable area (above the Dock, below the menu bar).
fn place_window(window: &tauri::WebviewWindow, corner: Corner) -> tauri::Result<()> {
    if let Some(m) = window.current_monitor()? {
        let wa = m.work_area();
        let area = Rect { x: wa.position.x, y: wa.position.y, w: wa.size.width as i32, h: wa.size.height as i32 };
        let size = window.outer_size()?;
        let margin = (CORNER_MARGIN * m.scale_factor()) as i32;
        let (x, y) = layout::window_position(corner, area, size.width as i32, size.height as i32, margin);
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

fn apply_settings(app: &AppHandle) {
    let s = current_settings(app);
    if let Some(w) = app.get_webview_window("main") {
        let _ = place_window(&w, s.corner);
    }
    let _ = app.emit("mascot-settings", &s);
    refresh_tray(app);
}

fn update_settings(app: &AppHandle, change: impl FnOnce(&mut Settings)) {
    {
        let state = app.state::<AppSettings>();
        let mut s = state.0.lock().unwrap();
        change(&mut s);
        if let Ok(h) = home(app) {
            let _ = settings::save(&h, &s);
        }
    }
    apply_settings(app);
}

fn notice(app: &AppHandle, text: impl Into<String>) {
    let _ = app.emit("mascot-notice", text.into());
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let s = current_settings(app);
    let installed = home(app).map(|h| installer::is_installed(&h)).unwrap_or(false);
    let status = MenuItem::with_id(app, "status",
        if installed { "Claude Code: connected" } else { "Claude Code: not connected" }, false, None::<&str>)?;
    let connect = if installed {
        MenuItem::with_id(app, "disconnect", "Disconnect Claude Code", true, None::<&str>)?
    } else {
        MenuItem::with_id(app, "connect", "Connect Claude Code", true, None::<&str>)?
    };
    let hide = MenuItem::with_id(app, "toggle-hidden", if s.hidden { "Show mascot" } else { "Hide mascot" }, true, None::<&str>)?;
    let corners = CORNERS
        .iter()
        .map(|(id, label, c)| CheckMenuItem::with_id(app, *id, *label, true, s.corner == *c, None::<&str>))
        .collect::<tauri::Result<Vec<_>>>()?;
    let corner_refs: Vec<&dyn IsMenuItem<Wry>> = corners.iter().map(|i| i as &dyn IsMenuItem<Wry>).collect();
    let sizes = SIZES
        .iter()
        .map(|(id, label, px)| CheckMenuItem::with_id(app, *id, *label, true, s.size == *px, None::<&str>))
        .collect::<tauri::Result<Vec<_>>>()?;
    let size_refs: Vec<&dyn IsMenuItem<Wry>> = sizes.iter().map(|i| i as &dyn IsMenuItem<Wry>).collect();
    let corner = Submenu::with_items(app, "Corner", true, &corner_refs)?;
    let size = Submenu::with_items(app, "Size", true, &size_refs)?;
    let auto_hide = CheckMenuItem::with_id(app, "auto-hide", "Hide while asleep", true, s.auto_hide, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Claude Puppet", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    Menu::with_items(app, &[
        &settings, &PredefinedMenuItem::separator(app)?,
        &status, &connect, &PredefinedMenuItem::separator(app)?,
        &hide, &corner, &size, &auto_hide, &PredefinedMenuItem::separator(app)?,
        &quit,
    ])
}

/// The small menu shown when right-clicking the mascot.
fn build_context_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let installed = home(app).map(|h| installer::is_installed(&h)).unwrap_or(false);
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "toggle-hidden", "Hide mascot", true, None::<&str>)?;
    let connect = if installed {
        MenuItem::with_id(app, "disconnect", "Disconnect Claude Code", true, None::<&str>)?
    } else {
        MenuItem::with_id(app, "connect", "Connect Claude Code", true, None::<&str>)?
    };
    let quit = MenuItem::with_id(app, "quit", "Quit Claude Puppet", true, None::<&str>)?;
    Menu::with_items(app, &[
        &settings, &hide, &connect, &PredefinedMenuItem::separator(app)?, &quit,
    ])
}

#[tauri::command]
fn show_context_menu(app: AppHandle, window: tauri::Window) -> Result<(), String> {
    let menu = build_context_menu(&app).map_err(|e| e.to_string())?;
    window.popup_menu(&menu).map_err(|e| e.to_string())
}

/// Brings this (normally background) app to the front so its window isn't left behind others.
fn bring_to_front() {
    #[cfg(target_os = "macos")]
    if let Some(mtm) = objc2::MainThreadMarker::new() {
        #[allow(deprecated)]
        objc2_app_kit::NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
    }
}

/// Opens the settings window, or focuses it if it is already open. While it is open
/// the app shows in the Dock so the window can take focus; it goes back to being a
/// background accessory when the window closes.
fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = app.run_on_main_thread(bring_to_front);
        let _ = w.set_focus();
        return;
    }
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
    let built = tauri::WebviewWindowBuilder::new(app, "settings", tauri::WebviewUrl::App("settings.html".into()))
        .title("Claude Puppet Settings")
        .inner_size(560.0, 820.0)
        .resizable(false)
        .center()
        .build();
    match built {
        Ok(w) => {
            let app2 = app.clone();
            w.on_window_event(move |e| {
                if let tauri::WindowEvent::Destroyed = e {
                    #[cfg(target_os = "macos")]
                    let _ = app2.set_activation_policy(tauri::ActivationPolicy::Accessory);
                }
            });
            let _ = app.run_on_main_thread(bring_to_front);
            let _ = w.set_focus();
        }
        Err(e) => notice(app, format!("Couldn't open Settings: {e}")),
    }
}

/// Fields the settings window may change; anything left out stays as it is.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsPatch {
    corner: Option<Corner>,
    size: Option<u32>,
    auto_hide: Option<bool>,
    hidden: Option<bool>,
    mascot: Option<String>,
    animation: Option<settings::Animation>,
}

#[tauri::command]
fn set_settings(app: AppHandle, patch: SettingsPatch) -> Settings {
    update_settings(&app, |s| {
        if let Some(v) = patch.corner { s.corner = v; }
        if let Some(v) = patch.size { s.size = v.clamp(settings::MIN_SIZE, settings::MAX_SIZE); }
        if let Some(v) = patch.auto_hide { s.auto_hide = v; }
        if let Some(v) = patch.hidden { s.hidden = v; }
        if let Some(v) = patch.mascot { s.mascot = v; }
        if let Some(v) = patch.animation { s.animation = v; }
    });
    current_settings(&app)
}

#[tauri::command]
fn desktop_status() -> bool {
    desktop_monitor::claude_running()
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

fn refresh_tray(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("main"), build_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "connect" => match mascot_install(app.clone()) {
            Ok(()) => notice(app, "Connected! Start a new Claude Code session to see me react."),
            Err(e) => notice(app, e),
        },
        "disconnect" => match mascot_uninstall(app.clone()) {
            Ok(()) => notice(app, "Disconnected from Claude Code."),
            Err(e) => notice(app, e),
        },
        "toggle-hidden" => update_settings(app, |s| s.hidden = !s.hidden),
        "auto-hide" => update_settings(app, |s| s.auto_hide = !s.auto_hide),
        "quit" => app.exit(0),
        "settings" => open_settings(app),
        other => {
            if let Some((_, _, c)) = CORNERS.iter().find(|(cid, _, _)| *cid == other) {
                let c = *c;
                update_settings(app, |s| s.corner = c);
            } else if let Some((_, _, px)) = SIZES.iter().find(|(sid, _, _)| *sid == other) {
                let px = *px;
                update_settings(app, |s| s.size = px);
            }
        }
    }
    refresh_tray(app);
}

/// The window ignores the mouse except while the cursor is over the mascot,
/// so hovering can show the usage panel without blocking clicks elsewhere.
fn spawn_hover_watch(app: AppHandle) {
    std::thread::spawn(move || {
        let mut was_inside = false;
        // Window geometry only changes with settings (corner, size) or a display change, so it is
        // cached; each poll then asks only for the cursor position.
        let mut cached: Option<(settings::Corner, u32, Rect, f64, std::time::Instant)> = None;
        let mut delay = HOVER_POLL;
        loop {
            std::thread::sleep(delay);
            let Some(w) = app.get_webview_window("main") else { continue };
            let s = current_settings(&app);
            if s.hidden && !was_inside {
                delay = HIDDEN_POLL; // nothing to hover while hidden
                continue;
            }
            let stale = cached.as_ref().is_none_or(|(c, size, _, _, at)| {
                *c != s.corner || *size != s.size || at.elapsed() > GEOMETRY_REFRESH
            });
            if stale {
                if let (Ok(p), Ok(sz), Ok(scale)) = (w.outer_position(), w.outer_size(), w.scale_factor()) {
                    let win = Rect { x: p.x, y: p.y, w: sz.width as i32, h: sz.height as i32 };
                    let size = (s.size as f64 * scale) as i32;
                    let rect = layout::mascot_rect(s.corner, win, size, (MASCOT_INSET * scale) as i32);
                    cached = Some((s.corner, s.size, rect, scale, std::time::Instant::now()));
                }
            }
            let Some((_, _, rect, scale, _)) = cached else { continue };
            let Ok(c) = app.cursor_position() else { continue };
            let inside = !s.hidden && rect.contains(c.x, c.y);
            delay = layout::hover_delay(layout::distance_to(&rect, c.x, c.y), scale);
            if inside != was_inside {
                let _ = w.set_ignore_cursor_events(!inside);
                let _ = app.emit("mascot-hover", inside);
                was_inside = inside;
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            reopen(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .manage(BridgeError(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            bridge_error, mascot_install, mascot_uninstall, mascot_status, get_settings,
            power::low_power, show_context_menu,
            set_settings, desktop_status, quit_app
        ])
        .setup(|app| {
            // No Dock icon and no app switcher entry: the mascot is an accessory overlay.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();
            let loaded = home(&handle).map(|h| settings::load(&h)).unwrap_or_default();
            app.manage(AppSettings(Mutex::new(loaded)));

            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().expect("app icon").clone())
                .tooltip("Claude Puppet")
                .menu(&build_menu(&handle)?)
                .show_menu_on_left_click(true)
                .build(app)?;

            let window = app.get_webview_window("main").expect("main window");
            window.set_ignore_cursor_events(true)?;
            apply_settings(&handle);
            spawn_hover_watch(handle.clone());
            power::spawn_watch(handle.clone());
            start_event_bridge(&handle);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Opening the app from Finder, Spotlight or the Dock while it runs sends a
            // "reopen" event (no second process): bring the mascot back and open Settings.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                reopen(app);
            }
        });
}

/// Reopening the app shows the mascot again and opens Settings.
fn reopen(app: &AppHandle) {
    update_settings(app, |s| s.hidden = false);
    open_settings(app);
}
