// ============================================================
// DevForge — Clipboard & Text Processing Module (Features 041–050)
// ============================================================

use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};
use std::collections::VecDeque;
use std::sync::Mutex;
use regex::Regex;

// ============================================================
// 041 - Multi-Format Structural Clipboard Stack
// In-memory clipboard history stack
// ============================================================

const MAX_CLIP_HISTORY: usize = 50;

struct ClipboardStack {
    items: VecDeque<String>,
}

lazy_static::lazy_static! {
    static ref CLIPBOARD: Mutex<ClipboardStack> = Mutex::new(ClipboardStack {
        items: VecDeque::with_capacity(MAX_CLIP_HISTORY),
    });
}

pub fn clipboard_stack(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed == "list" || trimmed == "ls" {
        let clip = CLIPBOARD.lock().unwrap();
        if clip.items.is_empty() {
            return Ok(CommandExecutionResult {
                command_id: "clip.stack".into(),
                title: "Clipboard History".into(),
                output: "No items in clipboard history.".into(),
                status: CommandExecutionStatus::Info,
                summary: "History is empty".into(),
            });
        }
        let mut output = String::new();
        for (i, item) in clip.items.iter().rev().enumerate() {
            let preview: String = item.chars().take(120).collect();
            let suffix = if item.len() > 120 { "..." } else { "" };
            output.push_str(&format!("{}. {}{}\n", i + 1, preview, suffix));
        }
        return Ok(CommandExecutionResult {
            command_id: "clip.stack".into(),
            title: format!("Clipboard History ({} items)", clip.items.len()),
            output,
            status: CommandExecutionStatus::Success,
            summary: format!("{} items in history", clip.items.len()),
        });
    }

    if trimmed == "clear" {
        let mut clip = CLIPBOARD.lock().unwrap();
        let count = clip.items.len();
        clip.items.clear();
        return Ok(CommandExecutionResult {
            command_id: "clip.stack".into(),
            title: "History Cleared".into(),
            output: format!("Cleared {} items.", count),
            status: CommandExecutionStatus::Success,
            summary: "History cleared".into(),
        });
    }

    if let Some(idx_str) = trimmed.strip_prefix("get ") {
        if let Ok(idx) = idx_str.trim().parse::<usize>() {
            let clip = CLIPBOARD.lock().unwrap();
            let items: Vec<&String> = clip.items.iter().rev().collect();
            if idx > 0 && idx <= items.len() {
                return Ok(CommandExecutionResult {
                    command_id: "clip.stack".into(),
                    title: format!("Clipboard #{}", idx),
                    output: items[idx - 1].clone(),
                    status: CommandExecutionStatus::Success,
                    summary: "Retrieved from history".into(),
                });
            }
        }
        return Ok(CommandExecutionResult {
            command_id: "clip.stack".into(),
            title: "Invalid Index".into(),
            output: format!("Index '{}' out of range.", idx_str),
            status: CommandExecutionStatus::Error,
            summary: "Invalid history index".into(),
        });
    }

    // Push new item to clipboard stack
    let mut clip = CLIPBOARD.lock().unwrap();
    if clip.items.len() >= MAX_CLIP_HISTORY {
        clip.items.pop_front();
    }
    clip.items.push_back(trimmed.to_string());

    Ok(CommandExecutionResult {
        command_id: "clip.stack".into(),
        title: "Clipboard Push".into(),
        output: format!("Pushed ({} chars). Use 'list' to view or 'get N' to retrieve.", trimmed.len()),
        status: CommandExecutionStatus::Success,
        summary: "Item added to clipboard history".into(),
    })
}

// ============================================================
// 042 - Semantic Contextual Data Classifier
// ============================================================
pub fn classify_text(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.classify".into(),
            title: "Classify Text".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste content to classify its semantic type.".into(),
        });
    }

    let mut categories: Vec<String> = Vec::new();

    // JSON detection
    if trimmed.trim().starts_with('{') || trimmed.trim().starts_with('[') {
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            categories.push("JSON".into());
        }
    }

    // URL detection
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        categories.push("URL".into());
    }

    // IP address detection
    let ip_re = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$").unwrap();
    if ip_re.is_match(trimmed) {
        categories.push("IP Address".into());
    }

    // Email detection
    let email_re = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if email_re.is_match(trimmed) {
        categories.push("Email".into());
    }

    // JWT detection
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() == 3 && parts[0].len() > 10 {
        categories.push("JWT Token".into());
    }

    // Base64 detection
    let b64_re = Regex::new(r"^[A-Za-z0-9+/=]{10,}$").unwrap();
    if b64_re.is_match(trimmed) {
        categories.push("Base64".into());
    }

    // Hex color detection
    let hex_re = Regex::new(r"^#[0-9a-fA-F]{3,8}$").unwrap();
    if hex_re.is_match(trimmed) {
        categories.push("Hex Color".into());
    }

    // Number detection
    if trimmed.parse::<f64>().is_ok() {
        categories.push("Number".into());
    }

    // UUID detection
    let uuid_re = Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$").unwrap();
    if uuid_re.is_match(trimmed) {
        categories.push("UUID".into());
    }

    // SQL detection
    let sql_keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "ALTER", "DROP", "FROM", "WHERE"];
    let upper = trimmed.to_uppercase();
    if sql_keywords.iter().any(|kw| upper.contains(kw)) {
        categories.push("SQL".into());
    }

    // HTML/XML detection
    if trimmed.contains('<') && trimmed.contains('>') {
        categories.push("Markup".into());
    }

    // Shell command detection
    let shell_chars = ['|', '>', '<', '&', '$', '`'];
    if trimmed.contains(' ') && shell_chars.iter().any(|c| trimmed.contains(*c)) {
        categories.push("Shell Command".into());
    }

    // General classification by length
    let word_count = trimmed.split_whitespace().count();
    if word_count > 50 {
        categories.push("Long Text".into());
    } else if word_count > 10 {
        categories.push("Short Text".into());
    } else {
        categories.push("Snippet".into());
    }

    if categories.is_empty() {
        categories.push("Plain Text".into());
    }

    let output = format!("Detected categories ({}):\n  {}", categories.len(), categories.join("\n  "));

    Ok(CommandExecutionResult {
        command_id: "clip.classify".into(),
        title: "Classification Result".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: categories.join(", "),
    })
}

// ============================================================
// 043 - Sticky Snippet Injection Board
// ============================================================

struct SnippetBoard {
    snippets: Vec<(String, String)>,
}

lazy_static::lazy_static! {
    static ref SNIPPETS: Mutex<SnippetBoard> = Mutex::new(SnippetBoard {
        snippets: Vec::new(),
    });
}

pub fn snippet_board(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        let board = SNIPPETS.lock().unwrap();
        if board.snippets.is_empty() {
            return Ok(CommandExecutionResult {
                command_id: "clip.snippets".into(),
                title: "Snippet Board".into(),
                output: String::new(),
                status: CommandExecutionStatus::Info,
                summary: "Commands: add:<label>:<value> | get:<label> | del:<label> | list | clear".into(),
            });
        }
        let mut output = String::from("Sticky snippets:\n");
        for (i, (label, value)) in board.snippets.iter().enumerate() {
            let preview: String = value.chars().take(60).collect();
            let suffix = if value.len() > 60 { "..." } else { "" };
            output.push_str(&format!("{}. [{}] {}{}\n", i + 1, label, preview, suffix));
        }
        return Ok(CommandExecutionResult {
            command_id: "clip.snippets".into(),
            title: format!("Snippet Board ({} items)", board.snippets.len()),
            output,
            status: CommandExecutionStatus::Success,
            summary: format!("{} snippets stored", board.snippets.len()),
        });
    }

    let (cmd, rest) = if let Some(pos) = trimmed.find(':') {
        (trimmed[..pos].trim().to_lowercase(), trimmed[pos + 1..].trim())
    } else {
        (trimmed.to_lowercase(), "")
    };

    match cmd.as_str() {
        "add" | "set" => {
            if let Some(pos) = rest.find(':') {
                let label = rest[..pos].trim();
                let value = rest[pos + 1..].trim();
                let mut board = SNIPPETS.lock().unwrap();
                if let Some(existing) = board.snippets.iter_mut().find(|(l, _)| l == label) {
                    existing.1 = value.to_string();
                } else {
                    board.snippets.push((label.to_string(), value.to_string()));
                }
                Ok(CommandExecutionResult {
                    command_id: "clip.snippets".into(),
                    title: "Snippet Added".into(),
                    output: format!("[{}] stored ({} chars)", label, value.len()),
                    status: CommandExecutionStatus::Success,
                    summary: format!("Snippet '{}' saved", label),
                })
            } else {
                Err(AppError::Internal("Format: add:<label>:<value>".into()))
            }
        }
        "get" => {
            let label = rest;
            let board = SNIPPETS.lock().unwrap();
            match board.snippets.iter().find(|(l, _)| l == label) {
                Some((_, value)) => Ok(CommandExecutionResult {
                    command_id: "clip.snippets".into(),
                    title: format!("Snippet: {}", label),
                    output: value.clone(),
                    status: CommandExecutionStatus::Success,
                    summary: format!("Retrieved '{}'", label),
                }),
                None => Ok(CommandExecutionResult {
                    command_id: "clip.snippets".into(),
                    title: "Not Found".into(),
                    output: format!("No snippet '{}'", label),
                    status: CommandExecutionStatus::Error,
                    summary: "Snippet not found".into(),
                }),
            }
        }
        "del" | "delete" | "remove" => {
            let label = rest;
            let mut board = SNIPPETS.lock().unwrap();
            let len_before = board.snippets.len();
            board.snippets.retain(|(l, _)| l != label);
            if board.snippets.len() < len_before {
                Ok(CommandExecutionResult {
                    command_id: "clip.snippets".into(),
                    title: "Snippet Deleted".into(),
                    output: format!("Deleted '{}'", label),
                    status: CommandExecutionStatus::Success,
                    summary: format!("Snippet '{}' removed", label),
                })
            } else {
                Ok(CommandExecutionResult {
                    command_id: "clip.snippets".into(),
                    title: "Not Found".into(),
                    output: format!("No snippet '{}'", label),
                    status: CommandExecutionStatus::Error,
                    summary: "Snippet not found".into(),
                })
            }
        }
        "list" | "ls" => {
            let board = SNIPPETS.lock().unwrap();
            let mut output = String::new();
            for (i, (label, value)) in board.snippets.iter().enumerate() {
                let preview: String = value.chars().take(80).collect();
                let suffix = if value.len() > 80 { "..." } else { "" };
                output.push_str(&format!("{}. [{}] {}{}\n", i + 1, label, preview, suffix));
            }
            Ok(CommandExecutionResult {
                command_id: "clip.snippets".into(),
                title: format!("Snippets ({})", board.snippets.len()),
                output,
                status: CommandExecutionStatus::Success,
                summary: format!("{} snippets", board.snippets.len()),
            })
        }
        "clear" => {
            let mut board = SNIPPETS.lock().unwrap();
            let count = board.snippets.len();
            board.snippets.clear();
            Ok(CommandExecutionResult {
                command_id: "clip.snippets".into(),
                title: "Snippets Cleared".into(),
                output: format!("Cleared {} snippets.", count),
                status: CommandExecutionStatus::Success,
                summary: "All snippets cleared".into(),
            })
        }
        _ => Ok(CommandExecutionResult {
            command_id: "clip.snippets".into(),
            title: "Unknown Command".into(),
            output: "Commands: add:<label>:<value> | get:<label> | del:<label> | list | clear".into(),
            status: CommandExecutionStatus::Info,
            summary: "See output for usage".into(),
        }),
    }
}

// ============================================================
// 044 - Bulk Content Merger & Splitter Suite
// ============================================================
pub fn merge_split(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.merge-split".into(),
            title: "Merge / Split".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Prefix:\nmerge:<separator>: merge lines with separator\nsplit:<separator>: split text by separator (one per line)".into(),
        });
    }

    let (mode, body) = if let Some(pos) = trimmed.find(':') {
        (trimmed[..pos].trim().to_lowercase(), trimmed[pos + 1..].trim())
    } else {
        return Err(AppError::Internal("Format: <mode>:<separator>:<text>".into()));
    };

    let (separator, text) = if let Some(pos) = body.find(':') {
        (body[..pos].trim().to_string(), body[pos + 1..].trim())
    } else {
        return Err(AppError::Internal("Format: <mode>:<separator>:<text>".into()));
    };

    let output = match mode.as_str() {
        "merge" | "join" => {
            let lines: Vec<&str> = text.lines().collect();
            lines.join(&separator)
        }
        "split" => {
            text.split(&separator)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n")
        }
        "lines" | "by-line" => {
            let sep_char = separator.chars().next().unwrap_or('\n');
            text.split(sep_char)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => return Err(AppError::Internal("Mode must be: merge, split, or lines".into())),
    };

    Ok(CommandExecutionResult {
        command_id: "clip.merge-split".into(),
        title: format!("{} Result", mode),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} completed", mode),
    })
}

// ============================================================
// 045 - RegEx Strip & Regex Find-and-Replace
// ============================================================
pub fn regex_transform(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.regex".into(),
            title: "Regex Transform".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Format (first line = pattern, second line = replacement, rest = text):\n<pattern>\n<replacement>\n<text>\n\nOr: strip:<pattern> to remove matches.".into(),
        });
    }

    let lines: Vec<&str> = trimmed.splitn(3, '\n').collect();
    if lines.len() < 3 {
        return Ok(CommandExecutionResult {
            command_id: "clip.regex".into(),
            title: "Regex Transform".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Format:\n<pattern>\n<replacement>\n<text>\n\nUse $1, $2 etc for capture groups.".into(),
        });
    }

    let pattern = lines[0].trim();
    let replacement = lines[1];
    let text = lines[2];

    let re = Regex::new(pattern)
        .map_err(|e| AppError::Internal(format!("Invalid regex: {}", e)))?;
    let output = re.replace_all(text, replacement).to_string();

    Ok(CommandExecutionResult {
        command_id: "clip.regex".into(),
        title: "Regex Result".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("Replacements: {}", re.find_iter(text).count()),
    })
}

// Handle strip: separately
pub fn regex_strip(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    let lines: Vec<&str> = trimmed.splitn(2, '\n').collect();
    if lines.len() < 2 {
        return Ok(CommandExecutionResult {
            command_id: "clip.regex".into(),
            title: "Regex Strip".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Format:\nstrip:<pattern>\n<text>".into(),
        });
    }

    let pattern = lines[0].trim();
    let text = lines[1];

    let pattern = if let Some(p) = pattern.strip_prefix("strip:") { p.trim() } else { pattern };

    let re = Regex::new(pattern)
        .map_err(|e| AppError::Internal(format!("Invalid regex: {}", e)))?;
    let output = re.replace_all(text, "").to_string();
    let count = re.find_iter(text).count();

    Ok(CommandExecutionResult {
        command_id: "clip.regex".into(),
        title: "Regex Strip".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} matches removed", count),
    })
}

// ============================================================
// 046 - Text Case Normalizer Engine
// ============================================================
pub fn case_normalize(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.case".into(),
            title: "Case Normalize".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste text. Output shows:\ncamelCase | snake_case | kebab-case | PASCAL_CASE | UPPERCASE | lowercase".into(),
        });
    }

    // Clean up the text: remove extra spaces, normalize
    let words: Vec<&str> = trimmed.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .collect();

    if words.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.case".into(),
            title: "Normalize".into(),
            output: "No alphanumeric words found.".into(),
            status: CommandExecutionStatus::Error,
            summary: "No words to normalize".into(),
        });
    }

    let lower_words: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let upper_words: Vec<String> = words.iter().map(|w| w.to_uppercase()).collect();

    let camel = {
        let mut s = lower_words[0].clone();
        for w in &lower_words[1..] {
            let mut chars = w.chars();
            if let Some(c) = chars.next() {
                s.push(c.to_ascii_uppercase());
                s.push_str(chars.as_str());
            }
        }
        s
    };

    let pascal = {
        let mut s = String::new();
        for w in &lower_words {
            let mut chars = w.chars();
            if let Some(c) = chars.next() {
                s.push(c.to_ascii_uppercase());
                s.push_str(chars.as_str());
            }
        }
        s
    };

    let snake = lower_words.join("_");
    let screaming_snake = upper_words.join("_");
    let kebab = lower_words.join("-");
    let upper = upper_words.join(" ");
    let lower = lower_words.join(" ");

    let output = format!(
        "camelCase:      {}\nPascalCase:     {}\nsnake_case:     {}\nSCREAMING_SNAKE: {}\nkebab-case:     {}\nUPPERCASE:      {}\nlowercase:      {}",
        camel, pascal, snake, screaming_snake, kebab, upper, lower
    );

    Ok(CommandExecutionResult {
        command_id: "clip.case".into(),
        title: "Case Conversions".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "7 case formats generated".into(),
    })
}

// ============================================================
// 047 - Clipboard Diff Comparison Bridge
// ============================================================
pub fn clipboard_diff(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.diff".into(),
            title: "Clipboard Diff".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste two texts separated by '---' on its own line to diff them.".into(),
        });
    }

    let mut parts: Vec<String> = Vec::new();
    let mut buf = String::new();
    for line in trimmed.lines() {
        if line.trim() == "---" {
            parts.push(buf.trim().to_string());
            buf = String::new();
        } else {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    let last = buf.trim().to_string();
    if !last.is_empty() {
        parts.push(last);
    }

    if parts.len() < 2 {
        return Ok(CommandExecutionResult {
            command_id: "clip.diff".into(),
            title: "Diff".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Separate two texts with '---' on its own line.".into(),
        });
    }

    let left = &parts[0];
    let right = &parts[1];

    let left_lines: Vec<&str> = left.lines().collect();
    let right_lines: Vec<&str> = right.lines().collect();

    let mut output = String::new();
    let mut changes = 0;

    // Simple line diff
    let max_lines = left_lines.len().max(right_lines.len());
    for i in 0..max_lines {
        let l = left_lines.get(i).copied().unwrap_or("");
        let r = right_lines.get(i).copied().unwrap_or("");

        if l != r {
            changes += 1;
            if !l.is_empty() {
                output.push_str(&format!("- {} | {}\n", i + 1, l));
            }
            if !r.is_empty() {
                // Only show right if it's different and not empty when left is empty
                if !l.is_empty() || (l.is_empty() && !r.is_empty()) {
                    output.push_str(&format!("+ {} | {}\n", i + 1, r));
                }
            }
        }
    }

    if changes == 0 {
        output = "No differences found — both texts are identical.".into();
    }

    Ok(CommandExecutionResult {
        command_id: "clip.diff".into(),
        title: format!("Diff Result ({} change(s))", changes),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} line(s) differ", changes),
    })
}

// ============================================================
// 048 - Whitespace Sanitizer & Minifier
// ============================================================
pub fn whitespace_sanitize(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.whitespace".into(),
            title: "Whitespace Sanitizer".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste text. Prefix:\nminify: collapse all whitespace to single spaces\ntrim: strip leading/trailing whitespace per line\nsingle: collapse multiple blank lines to one".into(),
        });
    }

    let (mode, text) = if let Some(pos) = trimmed.find(':') {
        let m = trimmed[..pos].trim().to_lowercase();
        let t = trimmed[pos + 1..].trim();
        (m, t)
    } else {
        // Default: show all sanitization modes
        let minified: String = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
        let single_spaced: String = trimmed.lines()
            .map(|l| {
                let s = l.trim();
                s.split_whitespace().collect::<Vec<_>>().join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let no_blank_lines: String = trimmed.lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let original_len = trimmed.len();
        let minified_len = minified.len();
        let output = format!(
            "─── Original ({}) ───\n{}\n\n─── Minified ({}, -{}%) ───\n{}\n\n─── Single-Spaced ───\n{}\n\n─── No Blank Lines ───\n{}",
            original_len, trimmed, minified_len, (original_len.saturating_sub(minified_len)) * 100 / original_len.max(1),
            minified, single_spaced, no_blank_lines
        );
        return Ok(CommandExecutionResult {
            command_id: "clip.whitespace".into(),
            title: "Whitespace Analysis".into(),
            output,
            status: CommandExecutionStatus::Success,
            summary: format!("Original {}B → Minified {}B", original_len, minified_len),
        });
    };

    let output = match mode.as_str() {
        "minify" => {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        }
        "trim" => {
            text.lines()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join("\n")
        }
        "single" | "single-line" | "collapse" => {
            text.lines()
                .filter(|l| !l.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => {
            let result: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
            format!("Unknown mode '{}'. Defaulting to minify:\n{}", mode, result)
        }
    };

    Ok(CommandExecutionResult {
        command_id: "clip.whitespace".into(),
        title: format!("Whitespace {}", mode),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} applied", mode),
    })
}

// ============================================================
// 049 - Automated Clipboard Sensitive Data Redactor
// ============================================================
pub fn redact_data(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.redact".into(),
            title: "Redact Data".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste text to redact sensitive patterns:\n- API keys / tokens\n- Passwords\n- Credit card numbers\n- Email addresses\n- IP addresses\n- Private SSH keys".into(),
        });
    }

    let mut output = trimmed.to_string();
    let mut redactions: Vec<String> = Vec::new();

    // API keys: alphanumeric 20-64 char tokens after common prefixes
    let api_key_re = Regex::new(r#"(?i)(api[_-]?key|token|secret|apikey)\s*[:=]\s*['"]?([A-Za-z0-9_\-\.]{20,64})['"]?"#).unwrap();
    if api_key_re.is_match(&output) {
        output = api_key_re.replace_all(&output, "$1: [REDACTED_API_KEY]").to_string();
        redactions.push("API keys/tokens".into());
    }

    // Generic bearer tokens
    let bearer_re = Regex::new(r"(?i)(Bearer\s+)[A-Za-z0-9_\-\.]{20,200}").unwrap();
    if bearer_re.is_match(&output) {
        output = bearer_re.replace_all(&output, "$1[REDACTED_TOKEN]").to_string();
        redactions.push("Bearer tokens".into());
    }

    // Credit cards: 13-19 digits with optional separators
    let cc_re = Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b").unwrap();
    if cc_re.is_match(&output) {
        output = cc_re.replace_all(&output, "[REDACTED_CC]").to_string();
        redactions.push("Credit card numbers".into());
    }

    // Email addresses
    let email_re = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap();
    if email_re.is_match(&output) {
        let count = email_re.find_iter(&output).count();
        output = email_re.replace_all(&output, "[REDACTED_EMAIL]").to_string();
        redactions.push(format!("Email addresses (x{})", count));
    }

    // IP addresses
    let ip_re = Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap();
    if ip_re.is_match(&output) {
        let count = ip_re.find_iter(&output).count();
        output = ip_re.replace_all(&output, "[REDACTED_IP]").to_string();
        redactions.push(format!("IP addresses (x{})", count));
    }

    // Private SSH keys
    let ssh_re = Regex::new(r"-----BEGIN\s+(RSA|DSA|EC|OPENSSH)\s+PRIVATE\s+KEY-----[\s\S]*?-----END\s+(?:RSA|DSA|EC|OPENSSH)\s+PRIVATE\s+KEY-----").unwrap();
    if ssh_re.is_match(&output) {
        output = ssh_re.replace_all(&output, "[REDACTED_PRIVATE_KEY]").to_string();
        redactions.push("Private SSH keys".into());
    }

    // Password fields in JSON-like structures
    let pwd_re = Regex::new(r#"(?i)("password"|"passwd"|"pwd")\s*:\s*"[^"]{3,}"}"#).unwrap();
    if pwd_re.is_match(&output) {
        output = pwd_re.replace_all(&output, "$1: \"[REDACTED_PASSWORD]\"").to_string();
        redactions.push("Password fields".into());
    }

    // AWS keys
    let aws_re = Regex::new(r"(?i)(AKIA[0-9A-Z]{16}|[A-Za-z0-9+/=]{40})").unwrap();
    if aws_re.is_match(&output) {
        output = aws_re.replace_all(&output, "[REDACTED_AWS_KEY]").to_string();
        redactions.push("AWS access keys".into());
    }

    if redactions.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "clip.redact".into(),
            title: "No Sensitive Data Found".into(),
            output: trimmed.to_string(),
            status: CommandExecutionStatus::Success,
            summary: "No sensitive patterns detected".into(),
        });
    }

    let summary = format!("Redacted: {}", redactions.join(", "));

    Ok(CommandExecutionResult {
        command_id: "clip.redact".into(),
        title: format!("Redacted ({})", redactions.len()),
        output,
        status: CommandExecutionStatus::Success,
        summary,
    })
}

// ============================================================
// 050 - Multi-Item Queue Stack Sequence
// ============================================================

struct ClipQueue {
    items: VecDeque<String>,
}

lazy_static::lazy_static! {
    static ref QUEUE: Mutex<ClipQueue> = Mutex::new(ClipQueue {
        items: VecDeque::new(),
    });
}

pub fn clip_queue(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed == "list" || trimmed == "ls" {
        let queue = QUEUE.lock().unwrap();
        if queue.items.is_empty() {
            return Ok(CommandExecutionResult {
                command_id: "clip.queue".into(),
                title: "Queue Empty".into(),
                output: "No items in queue.\n\nCommands:\n  push:<text>  - add item to queue\n  pop          - remove and show first item\n  list         - show all items\n  clear        - empty the queue".into(),
                status: CommandExecutionStatus::Info,
                summary: "Queue is empty".into(),
            });
        }
        let mut output = String::new();
        for (i, item) in queue.items.iter().enumerate() {
            let preview: String = item.chars().take(100).collect();
            let suffix = if item.len() > 100 { "..." } else { "" };
            output.push_str(&format!("{}. {}{}\n", i + 1, preview, suffix));
        }
        return Ok(CommandExecutionResult {
            command_id: "clip.queue".into(),
            title: format!("Queue ({} items)", queue.items.len()),
            output,
            status: CommandExecutionStatus::Success,
            summary: format!("{} items queued", queue.items.len()),
        });
    }

    if trimmed == "pop" {
        let mut queue = QUEUE.lock().unwrap();
        match queue.items.pop_front() {
            Some(item) => Ok(CommandExecutionResult {
                command_id: "clip.queue".into(),
                title: "Queue Pop".into(),
                output: item,
                status: CommandExecutionStatus::Success,
                summary: format!("{} items remaining", queue.items.len()),
            }),
            None => Ok(CommandExecutionResult {
                command_id: "clip.queue".into(),
                title: "Queue Empty".into(),
                output: "No items to pop.".into(),
                status: CommandExecutionStatus::Info,
                summary: "Queue is empty".into(),
            }),
        }
    } else if trimmed == "clear" {
        let mut queue = QUEUE.lock().unwrap();
        let count = queue.items.len();
        queue.items.clear();
        Ok(CommandExecutionResult {
            command_id: "clip.queue".into(),
            title: "Queue Cleared".into(),
            output: format!("Cleared {} items.", count),
            status: CommandExecutionStatus::Success,
            summary: "Queue cleared".into(),
        })
    } else if let Some(value) = trimmed.strip_prefix("push:") {
        let value = value.trim();
        let mut queue = QUEUE.lock().unwrap();
        queue.items.push_back(value.to_string());
        Ok(CommandExecutionResult {
            command_id: "clip.queue".into(),
            title: "Queue Push".into(),
            output: format!("Pushed ({} chars). {} items in queue.", value.len(), queue.items.len()),
            status: CommandExecutionStatus::Success,
            summary: format!("Item added ({} total)", queue.items.len()),
        })
    } else if let Some(value) = trimmed.strip_prefix("push-front:") {
        let value = value.trim();
        let mut queue = QUEUE.lock().unwrap();
        queue.items.push_front(value.to_string());
        Ok(CommandExecutionResult {
            command_id: "clip.queue".into(),
            title: "Queue Push Front".into(),
            output: format!("Pushed to front ({} chars). {} items in queue.", value.len(), queue.items.len()),
            status: CommandExecutionStatus::Success,
            summary: format!("Item added to front ({} total)", queue.items.len()),
        })
    } else {
        Ok(CommandExecutionResult {
            command_id: "clip.queue".into(),
            title: "Unknown Command".into(),
            output: "Commands:\n  push:<text>     - add to back\n  push-front:<text> - add to front\n  pop              - remove from front\n  list             - show all\n  clear            - empty queue".into(),
            status: CommandExecutionStatus::Info,
            summary: "See output for usage".into(),
        })
    }
}