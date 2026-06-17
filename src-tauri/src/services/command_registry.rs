use crate::models::{CommandAction, CommandCategory};

pub struct CommandRegistry {
    commands: Vec<CommandAction>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: core_commands(),
        }
    }

    pub fn commands(&self) -> &[CommandAction] {
        &self.commands
    }
}

fn core_commands() -> Vec<CommandAction> {
    vec![
        CommandAction {
            id: "data.format-json".into(),
            title: "Format JSON".into(),
            subtitle: "Pretty-print or minify JSON payloads locally.".into(),
            category: CommandCategory::Data,
            tags: vec!["json".into(), "formatter".into(), "payload".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "data.minify-json".into(),
            title: "Minify JSON".into(),
            subtitle: "Compress JSON payloads into a single line locally.".into(),
            category: CommandCategory::Data,
            tags: vec!["json".into(), "minify".into(), "payload".into()],
            shortcut: None,
            accepts_input: true,
        },
    ]
}
