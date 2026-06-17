mod app;
mod commands;
mod error;
mod models;
mod services;
mod state;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let state = state::AppState::new(handle)?;
            app.manage(state);
            #[cfg(desktop)]
            app::overlay::register_overlay_shortcut(app)?;
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
            commands::save_workspace_profile,
            commands::search_command_history,
            commands::toggle_pinned_module,
            commands::reload_extensions,
            commands::export_configuration_snapshot,
            commands::import_configuration,
            commands::write_text_file,
            commands::read_text_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevForge");
}
