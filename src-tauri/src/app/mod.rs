use crate::models::CommandCategory;

pub const DEFAULT_PROFILE_ID: &str = "default";

pub fn default_profile_name() -> &'static str {
    "Default Workspace"
}

pub fn default_profile_categories() -> Vec<CommandCategory> {
    vec![
        CommandCategory::Core,
        CommandCategory::Data,
        CommandCategory::Clipboard,
        CommandCategory::System,
    ]
}
