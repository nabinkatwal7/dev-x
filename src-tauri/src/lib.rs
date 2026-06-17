mod app;
mod commands;
mod error;
mod lib_fallback;
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap_app,
            commands::update_app_settings,
            commands::record_command_execution,
            commands::set_active_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevForge");
}
