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
            let state = state::AppState::new(handle);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::bootstrap_app])
        .run(tauri::generate_context!())
        .expect("error while running DevForge");
}
