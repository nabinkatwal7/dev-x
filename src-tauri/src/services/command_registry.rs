use crate::{
    app,
    models::{CommandAction, CommandCategory, WorkspaceProfile},
};

pub struct CommandRegistry {
    commands: Vec<CommandAction>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: seed_commands(),
        }
    }

    pub fn commands(&self) -> &[CommandAction] {
        &self.commands
    }

    pub fn default_profile(&self) -> WorkspaceProfile {
        WorkspaceProfile {
            id: app::DEFAULT_PROFILE_ID.to_string(),
            name: app::default_profile_name().to_string(),
            enabled_categories: vec![
                CommandCategory::Core,
                CommandCategory::Data,
                CommandCategory::Clipboard,
                CommandCategory::System,
            ],
        }
    }
}

fn seed_commands() -> Vec<CommandAction> {
    vec![
        CommandAction {
            id: "core.open-command-palette".into(),
            title: "Open Command Palette".into(),
            subtitle: "Primary keyboard-first launcher overlay.".into(),
            category: CommandCategory::Core,
            tags: vec!["launcher".into(), "overlay".into(), "omnibar".into()],
            shortcut: Some("Alt+Space".into()),
        },
        CommandAction {
            id: "data.format-json".into(),
            title: "Format JSON".into(),
            subtitle: "Pretty-print or minify JSON payloads locally.".into(),
            category: CommandCategory::Data,
            tags: vec!["json".into(), "formatter".into(), "payload".into()],
            shortcut: None,
        },
        CommandAction {
            id: "clipboard.history".into(),
            title: "Clipboard History".into(),
            subtitle: "Inspect and replay structured clipboard items.".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["clipboard".into(), "history".into(), "stack".into()],
            shortcut: Some("Alt+Shift+V".into()),
        },
        CommandAction {
            id: "system.port-monitor".into(),
            title: "Port Monitor".into(),
            subtitle: "Track listening local ports and owning processes.".into(),
            category: CommandCategory::System,
            tags: vec!["ports".into(), "process".into(), "network".into()],
            shortcut: None,
        },
    ]
}
