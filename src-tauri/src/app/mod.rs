pub mod overlay;

use crate::models::CommandCategory;

pub const DEFAULT_PROFILE_ID: &str = "default";
pub const MAIN_WINDOW_LABEL: &str = "main";

pub fn module_window_label(command_id: &str) -> String {
    format!("module-{}", command_id.replace(|char: char| !char.is_ascii_alphanumeric(), "-"))
}

pub fn default_profile_name() -> &'static str {
    "Default Workspace"
}

pub fn default_profile_categories() -> Vec<CommandCategory> {
    vec![CommandCategory::Data]
}

pub fn default_command_ids() -> Vec<&'static str> {
    vec!["data.format-json", "data.minify-json"]
}

pub fn platform_default_hotkey() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Command+Space"
    }

    #[cfg(not(target_os = "macos"))]
    {
        "Alt+Space"
    }
}
