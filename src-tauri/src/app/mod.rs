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
    vec![
        CommandCategory::Data,
        CommandCategory::Network,
        CommandCategory::Crypto,
        CommandCategory::Clipboard,
    ]
}

pub fn default_command_ids() -> Vec<&'static str> {
    vec![
        "data.format-json",
        "data.minify-json",
        "data.schema-convert",
        "data.json-flatten",
        "data.json-unflatten",
        "data.sql-beautify",
        "data.escape",
        "data.unescape",
        "data.csv-table",
        "data.gen-types",
        "data.struct-diff",
        "data.url-parse",
        "data.path-eval",
        "net.port-mapper",
        "net.kill-process",
        "net.port-monitor",
        "net.hosts-edit",
        "net.tunnel-mgr",
        "net.curl-builder",
        "net.ping",
        "net.trace",
        "net.ip-discover",
        "net.domain-check",
        "net.subnet-sweep",
        "crypto.hash",
        "crypto.jwt",
        "crypto.cipher",
        "crypto.rsa-keygen",
        "crypto.base64",
        "crypto.hash-bench",
        "crypto.html-decode",
        "crypto.gen-token",
        "crypto.hmac",
        "crypto.vault",
        "clip.stack",
        "clip.classify",
        "clip.snippets",
        "clip.merge-split",
        "clip.regex",
        "clip.case",
        "clip.diff",
        "clip.whitespace",
        "clip.redact",
        "clip.queue",
    ]
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
