use tauri::{App, AppHandle, Manager, Window, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::app::MAIN_WINDOW_LABEL;

pub fn register_overlay_shortcut(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = overlay_shortcut();
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
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }

    if let WindowEvent::Focused(false) = event {
        let _ = window.hide();
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

fn overlay_shortcut() -> Shortcut {
    #[cfg(target_os = "macos")]
    {
        Shortcut::new(Some(Modifiers::SUPER), Code::Space)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Shortcut::new(Some(Modifiers::ALT), Code::Space)
    }
}
