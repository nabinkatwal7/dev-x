use std::sync::Arc;

use tauri::State;

use crate::{
    models::{
        AppSettings, BootstrapPayload, CommandExecutionResult, CommandHistoryEntry, ConfigurationSnapshot,
        ExecuteCommandPayload, FilePathPayload, FileTextPayload, ImportConfigurationPayload,
        RecordCommandExecutionPayload, SaveWorkspaceProfilePayload, SearchHistoryPayload,
        UpdateSettingsPayload,
    },
    services::command_executor::CommandExecutor,
    state::AppState,
};

#[tauri::command]
pub fn bootstrap_app(state: State<'_, AppState>) -> Result<BootstrapPayload, String> {
    let commands = state.commands();
    state
        .storage
        .load_bootstrap_payload(
            commands,
            false,
            state.extension_loader.directory().display().to_string(),
            state.extension_loader.summaries(),
        )
        .map_err(|error| error.to_string())
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
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<BootstrapPayload, String> {
    let profile = state
        .storage
        .set_active_profile(&profile_id)
        .map_err(|error| error.to_string())?;
    crate::app::overlay::update_overlay_shortcut(&app, &profile.launch_hotkey)
        .map_err(|error| error.to_string())?;

    let commands = state.commands();
    state
        .storage
        .load_bootstrap_payload(
            commands,
            false,
            state.extension_loader.directory().display().to_string(),
            state.extension_loader.summaries(),
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn execute_command(
    state: State<'_, AppState>,
    payload: ExecuteCommandPayload,
) -> Result<CommandExecutionResult, String> {
    if state.extension_loader.has_command(&payload.command_id) {
        return state
            .extension_loader
            .execute(&payload.command_id, &payload.input)
            .map_err(|error| error.to_string());
    }

    let executor: Arc<CommandExecutor> = state.command_executor.clone();
    tokio::task::spawn_blocking(move || executor.execute(payload))
        .await
        .map_err(|e| format!("execution panicked: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_overlay(app: tauri::AppHandle) -> Result<(), String> {
    crate::app::overlay::hide_overlay_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn show_overlay(app: tauri::AppHandle) -> Result<(), String> {
    crate::app::overlay::show_overlay_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_workspace_profile(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    payload: SaveWorkspaceProfilePayload,
) -> Result<BootstrapPayload, String> {
    let saved_profile = state
        .storage
        .save_profile(payload)
        .map_err(|error| error.to_string())?;

    let active_profile = state
        .storage
        .set_active_profile(&saved_profile.id)
        .map_err(|error| error.to_string())?;

    crate::app::overlay::update_overlay_shortcut(&app, &active_profile.launch_hotkey)
        .map_err(|error| error.to_string())?;

    let commands = state.commands();
    state
        .storage
        .load_bootstrap_payload(
            commands,
            false,
            state.extension_loader.directory().display().to_string(),
            state.extension_loader.summaries(),
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn search_command_history(
    state: State<'_, AppState>,
    payload: SearchHistoryPayload,
) -> Result<Vec<CommandHistoryEntry>, String> {
    state
        .storage
        .search_history(payload)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn toggle_pinned_module(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    command_id: String,
) -> Result<BootstrapPayload, String> {
    let commands = state.commands();
    let command = commands
        .iter()
        .find(|item| item.id == command_id)
        .ok_or_else(|| format!("unknown command id: {command_id}"))?;
    let should_close = crate::app::overlay::module_window_exists(&app, &command.id);
    let app_handle = app.clone();
    let command_id = command.id.clone();
    let command_title = command.title.clone();

    if should_close {
        state
            .storage
            .remove_pinned_module(&command_id)
            .map_err(|error| error.to_string())?;
        std::thread::spawn(move || {
            let _ = crate::app::overlay::close_module_window(&app_handle, &command_id);
        });
    } else {
        state
            .storage
            .upsert_pinned_module(&command_id, &command_title)
            .map_err(|error| error.to_string())?;
        std::thread::spawn(move || {
            let _ = crate::app::overlay::open_module_window(&app_handle, &command_id, &command_title);
        });
    }

    state
        .storage
        .load_bootstrap_payload(
            commands,
            false,
            state.extension_loader.directory().display().to_string(),
            state.extension_loader.summaries(),
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn reload_extensions(state: State<'_, AppState>) -> Result<BootstrapPayload, String> {
    state.extension_loader.refresh().map_err(|error| error.to_string())?;
    let commands = state.commands();
    state
        .storage
        .load_bootstrap_payload(
            commands,
            false,
            state.extension_loader.directory().display().to_string(),
            state.extension_loader.summaries(),
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn export_configuration_snapshot(
    state: State<'_, AppState>,
) -> Result<ConfigurationSnapshot, String> {
    state
        .storage
        .configuration_snapshot()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_configuration(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    payload: ImportConfigurationPayload,
) -> Result<BootstrapPayload, String> {
    let active_profile = state
        .storage
        .import_configuration(payload.snapshot)
        .map_err(|error| error.to_string())?;
    crate::app::overlay::update_overlay_shortcut(&app, &active_profile.launch_hotkey)
        .map_err(|error| error.to_string())?;

    let commands = state.commands();
    state
        .storage
        .load_bootstrap_payload(
            commands,
            false,
            state.extension_loader.directory().display().to_string(),
            state.extension_loader.summaries(),
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn write_text_file(state: State<'_, AppState>, payload: FileTextPayload) -> Result<(), String> {
    state
        .storage
        .write_text_file(&payload.path, &payload.contents)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn read_text_file(state: State<'_, AppState>, payload: FilePathPayload) -> Result<String, String> {
    state
        .storage
        .read_text_file(&payload.path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_system_metrics() -> Result<crate::models::SystemMetrics, String> {
    crate::services::system_metrics::get_metrics().map_err(|e| e.to_string())
}
