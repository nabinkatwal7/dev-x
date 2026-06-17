mod app;
mod commands;
mod error;
mod models;
mod services;
mod state;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let state = state::AppState::new(handle.clone())?;
            app.manage(state);
            #[cfg(desktop)]
            app::overlay::register_overlay_shortcut(app)?;
            #[cfg(desktop)]
            let _ = app::overlay::open_status_bar(&handle);
            #[cfg(desktop)]
            setup_tray(&handle)?;
            Ok(())
        })
        .on_window_event(app::overlay::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap_app,
            commands::update_app_settings,
            commands::record_command_execution,
            commands::set_active_profile,
            commands::execute_command,
            commands::hide_overlay,
            commands::show_overlay,
            commands::save_workspace_profile,
            commands::search_command_history,
            commands::toggle_pinned_module,
            commands::reload_extensions,
            commands::export_configuration_snapshot,
            commands::import_configuration,
            commands::write_text_file,
            commands::read_text_file,
            commands::get_system_metrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevForge");
}

#[cfg(desktop)]
fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "Show").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app).item(&show).separator().item(&quit).build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                let _ = app::overlay::show_overlay_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
