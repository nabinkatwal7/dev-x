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
    pub accepts_input: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceProfile {
    pub id: String,
    pub name: String,
    pub environment_tags: Vec<String>,
    pub enabled_categories: Vec<CommandCategory>,
    pub enabled_command_ids: Vec<String>,
    pub default_command_id: String,
    pub launch_hotkey: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppHealth {
    pub profile: WorkspaceProfile,
    pub command_count: usize,
    pub tray_ready: bool,
    pub storage_ready: bool,
    pub extension_directory: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub health: AppHealth,
    pub settings: AppSettings,
    pub profiles: Vec<WorkspaceProfile>,
    pub recent_history: Vec<CommandHistoryEntry>,
    pub command_usage: Vec<CommandUsageEntry>,
    pub pinned_modules: Vec<PinnedModule>,
    pub extensions: Vec<ScriptExtensionSummary>,
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
    pub input_text: String,
    pub executed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordCommandExecutionPayload {
    pub command_id: String,
    pub query_text: String,
    pub input_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsPayload {
    pub theme_mode: String,
    pub launch_hotkey: String,
    pub close_to_tray: bool,
    pub history_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCommandPayload {
    pub command_id: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecutionResult {
    pub command_id: String,
    pub title: String,
    pub output: String,
    pub status: CommandExecutionStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandExecutionStatus {
    Success,
    Error,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandUsageEntry {
    pub command_id: String,
    pub execution_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWorkspaceProfilePayload {
    pub id: Option<String>,
    pub name: String,
    pub environment_tags: Vec<String>,
    pub enabled_command_ids: Vec<String>,
    pub default_command_id: String,
    pub launch_hotkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHistoryPayload {
    pub query_text: String,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinnedModule {
    pub command_id: String,
    pub window_label: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptExtensionSummary {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub source_path: String,
    pub command_path: String,
    pub accepts_input: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationSnapshot {
    pub settings: AppSettings,
    pub profiles: Vec<WorkspaceProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportConfigurationPayload {
    pub snapshot: ConfigurationSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTextPayload {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePathPayload {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetrics {
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub memory_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub cpu_percent: f64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub uptime_secs: u64,
    pub process_count: usize,
}
