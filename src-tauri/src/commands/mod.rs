use tauri::State;

use crate::{
    models::{AppHealth, BootstrapPayload},
    state::AppState,
};

#[tauri::command]
pub async fn bootstrap_app(state: State<'_, AppState>) -> BootstrapPayload {
    let commands = state.command_registry.commands().to_vec();
    let profile = state.command_registry.default_profile();

    BootstrapPayload {
        health: AppHealth {
            profile,
            command_count: commands.len(),
            tray_ready: false,
            storage_ready: false,
        },
        commands,
    }
}
