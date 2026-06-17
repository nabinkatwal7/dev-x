use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandAction {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub category: CommandCategory,
    pub tags: Vec<String>,
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceProfile {
    pub id: String,
    pub name: String,
    pub enabled_categories: Vec<CommandCategory>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppHealth {
    pub profile: WorkspaceProfile,
    pub command_count: usize,
    pub tray_ready: bool,
    pub storage_ready: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub health: AppHealth,
    pub settings: AppSettings,
    pub profiles: Vec<WorkspaceProfile>,
    pub recent_history: Vec<CommandHistoryEntry>,
    pub commands: Vec<CommandAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandCategory {
    Core,
    Clipboard,
    Data,
    System,
    Crypto,
    Network,
    Filesystem,
    Ai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme_mode: String,
    pub launch_hotkey: String,
    pub close_to_tray: bool,
    pub history_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandHistoryEntry {
    pub id: i64,
    pub command_id: String,
    pub query_text: String,
    pub executed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordCommandExecutionPayload {
    pub command_id: String,
    pub query_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsPayload {
    pub theme_mode: String,
    pub launch_hotkey: String,
    pub close_to_tray: bool,
    pub history_limit: u32,
}
