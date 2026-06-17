use serde::Serialize;

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
    pub commands: Vec<CommandAction>,
}

#[derive(Debug, Clone, Serialize)]
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
