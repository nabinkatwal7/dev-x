use tauri::State;

use crate::{
    lib_fallback,
    models::{
        AppSettings, BootstrapPayload, CommandHistoryEntry, RecordCommandExecutionPayload,
        UpdateSettingsPayload, WorkspaceProfile,
    },
    state::AppState,
};

#[tauri::command]
pub fn bootstrap_app(state: State<'_, AppState>) -> BootstrapPayload {
    let commands = state.command_registry.commands().to_vec();
    state
        .storage
        .load_bootstrap_payload(commands, false)
        .unwrap_or_else(|_| lib_fallback::bootstrap_payload())
}

#[tauri::command]
pub fn update_app_settings(
    state: State<'_, AppState>,
    payload: UpdateSettingsPayload,
) -> Result<AppSettings, String> {
    state
        .storage
        .update_settings(payload)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn record_command_execution(
    state: State<'_, AppState>,
    payload: RecordCommandExecutionPayload,
) -> Result<Vec<CommandHistoryEntry>, String> {
    state
        .storage
        .record_command_execution(payload)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_active_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<WorkspaceProfile, String> {
    state
        .storage
        .set_active_profile(&profile_id)
        .map_err(|error| error.to_string())
}
