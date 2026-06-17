use crate::{
    app,
    models::{
        AppHealth, AppSettings, BootstrapPayload, CommandAction, CommandCategory, WorkspaceProfile,
    },
};

pub fn bootstrap_payload() -> BootstrapPayload {
    let commands = vec![
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
    ];

    BootstrapPayload {
        health: AppHealth {
            profile: WorkspaceProfile {
                id: app::DEFAULT_PROFILE_ID.into(),
                name: app::default_profile_name().into(),
                enabled_categories: app::default_profile_categories(),
                is_default: true,
            },
            command_count: commands.len(),
            tray_ready: false,
            storage_ready: false,
        },
        settings: AppSettings {
            theme_mode: "system".into(),
            launch_hotkey: "Alt+Space".into(),
            close_to_tray: false,
            history_limit: 50,
        },
        profiles: vec![WorkspaceProfile {
            id: app::DEFAULT_PROFILE_ID.into(),
            name: app::default_profile_name().into(),
            enabled_categories: app::default_profile_categories(),
            is_default: true,
        }],
        recent_history: vec![],
        commands,
    }
}
