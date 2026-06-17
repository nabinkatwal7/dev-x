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
    if window.label() == MAIN_WINDOW_LABEL {
        if let WindowEvent::Focused(false) = event {
            let _ = window.hide();
        }
        return;
    }

    if !window.label().starts_with("module-") {
        return;
    }

    if matches!(event, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
        let state = window.app_handle().state::<AppState>();
        let _ = state.storage.remove_pinned_module_by_window_label(window.label());
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

pub fn toggle_overlay_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            window.hide()?;
        } else {
            window.show()?;
            let _ = window.unminimize();
            let _ = window.center();
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
