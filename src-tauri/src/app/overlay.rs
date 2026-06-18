use tauri::{App, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, Window, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::{app::MAIN_WINDOW_LABEL, state::AppState};

pub fn register_overlay_shortcut(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = overlay_shortcut_from_string(crate::app::platform_default_hotkey())?;
    let toggle_shortcut = shortcut.clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, pressed_shortcut, event| {
                if pressed_shortcut == &toggle_shortcut && event.state() == ShortcutState::Pressed {
                    let _ = toggle_overlay_window(app);
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(shortcut)?;
    Ok(())
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    let label = window.label().to_string();
    let app = window.app_handle().clone();

    if label == STATUSBAR_LABEL {
        if matches!(event, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
            // Recreate the status bar if it gets destroyed
            let _ = std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = open_status_bar(&app);
            });
        }
        return;
    }

    if label == MAIN_WINDOW_LABEL {
        if let WindowEvent::Focused(false) = event {
            let _ = window.hide();
        }
        if matches!(event, WindowEvent::CloseRequested { .. }) {
            let _ = window.hide();
            return;
        }
        return;
    }

    if !label.starts_with("module-") {
        return;
    }

    if matches!(event, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
        let state = window.app_handle().state::<AppState>();
        let _ = state.storage.remove_pinned_module_by_window_label(&label);
    }
}

pub fn hide_overlay_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            window.hide()?;
        }
    }

    Ok(())
}

pub fn show_overlay_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.show()?;
        let _ = window.unminimize();
        let _ = window.maximize();
        window.set_focus()?;
    }
    Ok(())
}

pub fn toggle_overlay_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            window.hide()?;
        } else {
            window.show()?;
            let _ = window.unminimize();
            let _ = window.maximize();
            window.set_focus()?;
        }
    }

    Ok(())
}

pub fn module_window_exists(app: &AppHandle, command_id: &str) -> bool {
    let label = crate::app::module_window_label(command_id);
    app.get_webview_window(&label).is_some()
}

pub fn open_module_window(
    app: &AppHandle,
    command_id: &str,
    title: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let label = crate::app::module_window_label(command_id);
    if let Some(window) = app.get_webview_window(&label) {
        window.show()?;
        window.set_focus()?;
        return Ok(label);
    }

    let init_script = format!(
        "window.__DEVFORGE_PINNED_COMMAND__ = {};",
        serde_json::to_string(command_id)?
    );

    WebviewWindowBuilder::new(app, label.clone(), WebviewUrl::default())
        .title(title)
        .inner_size(780.0, 640.0)
        .min_inner_size(580.0, 420.0)
        .always_on_top(true)
        .resizable(true)
        .focused(true)
        .visible(true)
        .initialization_script(&init_script)
        .build()?;

    Ok(label)
}

pub fn close_module_window(app: &AppHandle, command_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let label = crate::app::module_window_label(command_id);
    if let Some(window) = app.get_webview_window(&label) {
        window.close()?;
    }
    Ok(())
}

pub fn update_overlay_shortcut(
    app: &AppHandle,
    shortcut: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed = overlay_shortcut_from_string(shortcut)?;
    app.global_shortcut().unregister_all()?;
    app.global_shortcut().register(parsed)?;
    Ok(())
}

fn overlay_shortcut_from_string(shortcut: &str) -> Result<Shortcut, Box<dyn std::error::Error>> {
    Ok(shortcut.parse::<Shortcut>()?)
}

pub const STATUSBAR_LABEL: &str = "status-bar";

pub fn open_status_bar(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if app.get_webview_window(STATUSBAR_LABEL).is_some() {
        return Ok(());
    }

    let init_script = "window.__DEVFORGE_STATUSBAR__ = true;";

    let window = WebviewWindowBuilder::new(app, STATUSBAR_LABEL, WebviewUrl::default())
        .title("")
        .inner_size(260.0, 40.0)
        .min_inner_size(120.0, 36.0)
        .always_on_top(true)
        .decorations(false)
        .resizable(false)
        .focused(false)
        .visible(true)
        .skip_taskbar(true)
        .initialization_script(init_script)
        .build()?;

    // Position at top-right of screen
    if let Some(monitor) = window.available_monitors()?.first() {
        let pos = monitor.position();
        let size = monitor.size();
        let w = 260.0;
        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: (size.width as i32 - w as i32 - 20).max(0),
            y: pos.y + 8,
        }))?;
    }

    Ok(())
}
