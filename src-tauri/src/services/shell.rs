use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};
use regex::Regex;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone)]
struct CheatSheetEntry {
    tool: &'static str,
    topic: &'static str,
    summary: &'static str,
    syntax: &'static str,
    flags: &'static [&'static str],
    examples: &'static [&'static str],
    keywords: &'static [&'static str],
}

#[derive(Clone)]
struct BlueprintEntry {
    key: &'static str,
    title: &'static str,
    summary: &'static str,
    commands: &'static [&'static str],
    notes: &'static [&'static str],
    keywords: &'static [&'static str],
}

#[derive(Clone)]
struct ExitCodeEntry {
    code: i32,
    platform: &'static str,
    meaning: &'static str,
    notes: &'static [&'static str],
    keywords: &'static [&'static str],
}

#[derive(Clone)]
struct CodeSnippet {
    name: String,
    language: String,
    content: String,
}

lazy_static::lazy_static! {
    static ref CODE_VAULT: Mutex<VecDeque<CodeSnippet>> = Mutex::new(VecDeque::new());
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").unwrap();
    static ref ANSI_OSC_REGEX: Regex = Regex::new(r"\x1B\][^\x07\x1B]*(?:\x07|\x1B\\)").unwrap();
}

const MAX_VAULT_ITEMS: usize = 200;

const CHEAT_SHEET: &[CheatSheetEntry] = &[
    CheatSheetEntry {
        tool: "tar",
        topic: "Create gzip archive",
        summary: "Create a compressed tarball from a file or directory.",
        syntax: "tar -czf archive.tar.gz <path>",
        flags: &["-c create", "-z gzip", "-f output file"],
        examples: &["tar -czf release.tar.gz dist", "tar -czf backup.tar.gz src package.json"],
        keywords: &["archive", "compress", "gzip", "backup", "package"],
    },
    CheatSheetEntry {
        tool: "tar",
        topic: "Extract archive",
        summary: "Extract a gzip-compressed tarball into the current or target directory.",
        syntax: "tar -xzf archive.tar.gz [-C target-dir]",
        flags: &["-x extract", "-z gzip", "-f input file", "-C target directory"],
        examples: &["tar -xzf release.tar.gz", "tar -xzf release.tar.gz -C ./tmp"],
        keywords: &["extract", "unpack", "untar", "restore"],
    },
    CheatSheetEntry {
        tool: "tar",
        topic: "List contents",
        summary: "Inspect archive contents without extracting files.",
        syntax: "tar -tzf archive.tar.gz",
        flags: &["-t list", "-z gzip", "-f input file"],
        examples: &["tar -tzf release.tar.gz"],
        keywords: &["inspect", "contents", "preview", "show"],
    },
    CheatSheetEntry {
        tool: "find",
        topic: "Find by name",
        summary: "Locate files or directories by name pattern.",
        syntax: "find <path> -name '<pattern>'",
        flags: &["-name case-sensitive pattern", "-iname case-insensitive pattern"],
        examples: &["find . -name '*.ts'", "find src -iname '*config*'"],
        keywords: &["search", "filename", "glob", "locate"],
    },
    CheatSheetEntry {
        tool: "find",
        topic: "Filter by type",
        summary: "Restrict matches to files, directories, or symlinks.",
        syntax: "find <path> -type [f|d|l]",
        flags: &["-type f regular file", "-type d directory", "-type l symlink"],
        examples: &["find . -type f -name '*.rs'", "find . -type d -name 'node_modules'"],
        keywords: &["type", "file", "directory", "symlink"],
    },
    CheatSheetEntry {
        tool: "find",
        topic: "Find by size or age",
        summary: "Match files using size thresholds or modification time windows.",
        syntax: "find <path> [-size +10M] [-mtime -7]",
        flags: &["-size +N larger than N", "-size -N smaller than N", "-mtime days since modification"],
        examples: &["find . -type f -size +100M", "find logs -type f -mtime -2"],
        keywords: &["size", "mtime", "old", "recent", "cleanup"],
    },
    CheatSheetEntry {
        tool: "find",
        topic: "Execute a command per match",
        summary: "Run another command for each file found.",
        syntax: "find <path> <filters> -exec <command> {} \\;",
        flags: &["-exec run command", "{} current match", "\\; command terminator"],
        examples: &["find . -name '*.log' -exec rm {} \\;", "find src -name '*.ts' -exec sed -n '1,5p' {} \\;"],
        keywords: &["exec", "loop", "batch", "per-file"],
    },
    CheatSheetEntry {
        tool: "sed",
        topic: "Print matching lines",
        summary: "Filter output to lines that match a pattern.",
        syntax: "sed -n '/pattern/p' <file>",
        flags: &["-n suppress default output", "p print matches"],
        examples: &["sed -n '/TODO/p' README.md", "git diff | sed -n '/^+/p'"],
        keywords: &["grep", "match", "print", "filter"],
    },
    CheatSheetEntry {
        tool: "sed",
        topic: "Replace text",
        summary: "Substitute one string or regex match for another.",
        syntax: "sed 's/<pattern>/<replacement>/g' <file>",
        flags: &["s substitute", "g global replacement", "-E extended regex"],
        examples: &["sed 's/foo/bar/g' app.txt", "sed -E 's/[0-9]+/[n]/g' log.txt"],
        keywords: &["replace", "substitute", "regex", "transform"],
    },
    CheatSheetEntry {
        tool: "sed",
        topic: "Edit files in place",
        summary: "Write replacements back to the source file.",
        syntax: "sed -i[.bak] 's/<pattern>/<replacement>/g' <file>",
        flags: &["-i edit in place", ".bak keep backup copy"],
        examples: &["sed -i.bak 's/localhost/127.0.0.1/g' .env", "sed -i 's/debug/info/g' app.log"],
        keywords: &["in-place", "edit", "backup", "rewrite"],
    },
    CheatSheetEntry {
        tool: "awk",
        topic: "Print selected columns",
        summary: "Extract fields from tabular or delimited text.",
        syntax: "awk '{print $1, $3}' <file>",
        flags: &["$N field index", "-F custom separator"],
        examples: &["awk '{print $1, $3}' access.log", "awk -F, '{print $1, $4}' data.csv"],
        keywords: &["columns", "fields", "csv", "extract"],
    },
    CheatSheetEntry {
        tool: "awk",
        topic: "Filter rows by condition",
        summary: "Keep only rows whose fields match an expression.",
        syntax: "awk '<condition> {print}' <file>",
        flags: &["== equals", "> greater than", "~ regex match"],
        examples: &["awk '$3 > 100 {print}' metrics.txt", "awk '$1 ~ /ERROR/ {print}' app.log"],
        keywords: &["filter", "condition", "rows", "match"],
    },
    CheatSheetEntry {
        tool: "awk",
        topic: "Aggregate values",
        summary: "Compute totals, counts, and derived output from structured text.",
        syntax: "awk '{sum += $N} END {print sum}' <file>",
        flags: &["BEGIN setup block", "END final block"],
        examples: &["awk '{sum += $2} END {print sum}' sizes.txt", "awk 'END {print NR}' access.log"],
        keywords: &["sum", "count", "aggregate", "report"],
    },
    CheatSheetEntry {
        tool: "chmod",
        topic: "Set numeric permissions",
        summary: "Apply rwx permissions using octal notation.",
        syntax: "chmod 755 <path>",
        flags: &["7 rwx", "6 rw-", "5 r-x", "4 r--"],
        examples: &["chmod 755 script.sh", "chmod 644 config.json"],
        keywords: &["permissions", "octal", "rwx", "mode"],
    },
    CheatSheetEntry {
        tool: "chmod",
        topic: "Set symbolic permissions",
        summary: "Add or remove permissions for user, group, or others.",
        syntax: "chmod u+x,g-w,o-r <path>",
        flags: &["u user", "g group", "o others", "+ add", "- remove", "= set exact"],
        examples: &["chmod u+x deploy.sh", "chmod go-rwx secret.txt"],
        keywords: &["symbolic", "execute", "restrict", "grant"],
    },
    CheatSheetEntry {
        tool: "chmod",
        topic: "Apply recursively",
        summary: "Change permissions across a whole directory tree.",
        syntax: "chmod -R 755 <dir>",
        flags: &["-R recursive"],
        examples: &["chmod -R 755 scripts", "find . -type f -name '*.sh' -exec chmod +x {} \\;"],
        keywords: &["recursive", "directory", "tree", "batch"],
    },
];

const GIT_BLUEPRINTS: &[BlueprintEntry] = &[
    BlueprintEntry {
        key: "rebase-interactive",
        title: "Interactive Rebase Last N Commits",
        summary: "Rewrite recent commits, reorder them, squash them, or edit messages.",
        commands: &["git log --oneline -n 8", "git rebase -i HEAD~4"],
        notes: &["Mark commits as pick, squash, fixup, reword, or edit in the todo list.", "Use git rebase --continue after resolving conflicts."],
        keywords: &["rebase", "interactive", "squash", "reword", "edit history"],
    },
    BlueprintEntry {
        key: "squash-last",
        title: "Squash Last Commits Into One",
        summary: "Condense several recent commits into a single commit before pushing.",
        commands: &["git rebase -i HEAD~3", "# change the second and later picks to squash or fixup"],
        notes: &["Use squash to combine messages or fixup to keep only the first message."],
        keywords: &["squash", "combine commits", "cleanup history"],
    },
    BlueprintEntry {
        key: "undo-keep",
        title: "Undo Last Commit Keep Changes",
        summary: "Move HEAD back while preserving your file changes locally.",
        commands: &["git reset --soft HEAD~1", "git status"],
        notes: &["Use --mixed instead of --soft if you want changes unstaged."],
        keywords: &["undo commit", "soft reset", "keep changes"],
    },
    BlueprintEntry {
        key: "fixup",
        title: "Create Fixup Commit Then Autosquash",
        summary: "Amend an older commit cleanly during rebase.",
        commands: &["git log --oneline", "git commit --fixup <commit-sha>", "git rebase -i --autosquash <base-sha>"],
        notes: &["Autosquash reorders the fixup commit next to its target automatically."],
        keywords: &["fixup", "autosquash", "amend previous commit"],
    },
    BlueprintEntry {
        key: "sync-fork",
        title: "Sync Branch With Main",
        summary: "Bring your current branch up to date with origin/main using rebase.",
        commands: &["git fetch origin", "git rebase origin/main"],
        notes: &["If the branch is already shared, use merge instead of rebase to avoid rewriting pushed history."],
        keywords: &["sync", "update branch", "fetch", "main", "origin"],
    },
    BlueprintEntry {
        key: "hard-reset-remote",
        title: "Reset Branch To Remote State",
        summary: "Discard local branch commits and match the remote exactly.",
        commands: &["git fetch origin", "git reset --hard origin/main"],
        notes: &["Destructive: this discards uncommitted and divergent local history on the current branch."],
        keywords: &["hard reset", "discard", "origin/main", "danger"],
    },
    BlueprintEntry {
        key: "recover-detached",
        title: "Recover From Detached HEAD",
        summary: "Turn detached work into a normal branch.",
        commands: &["git switch -c rescue/<topic>", "git log --oneline --decorate -n 5"],
        notes: &["If you already left detached HEAD, inspect git reflog to find the commit."],
        keywords: &["detached", "reflog", "recover", "rescue branch"],
    },
    BlueprintEntry {
        key: "rename-branch",
        title: "Rename Local And Remote Branch",
        summary: "Rename an existing branch and update the upstream tracking branch.",
        commands: &["git branch -m old-name new-name", "git push origin -u new-name", "git push origin --delete old-name"],
        notes: &["Coordinate with collaborators before deleting the old remote branch."],
        keywords: &["rename branch", "upstream", "push", "delete old remote"],
    },
];

const EXIT_CODES: &[ExitCodeEntry] = &[
    ExitCodeEntry {
        code: 0,
        platform: "unix/windows",
        meaning: "Success.",
        notes: &["The process completed without reporting an error."],
        keywords: &["success", "ok", "passed"],
    },
    ExitCodeEntry {
        code: 1,
        platform: "unix/windows",
        meaning: "General failure.",
        notes: &["Common catch-all for runtime, validation, or script errors."],
        keywords: &["general", "failure", "error"],
    },
    ExitCodeEntry {
        code: 2,
        platform: "unix/windows",
        meaning: "Misuse of shell builtins or invalid command usage.",
        notes: &["Often means bad flags, missing arguments, or syntax errors."],
        keywords: &["usage", "bad args", "syntax"],
    },
    ExitCodeEntry {
        code: 126,
        platform: "unix",
        meaning: "Command found but not executable.",
        notes: &["Check file permissions, shebangs, and execution policy."],
        keywords: &["not executable", "permission"],
    },
    ExitCodeEntry {
        code: 127,
        platform: "unix",
        meaning: "Command not found.",
        notes: &["Usually PATH is missing the binary or the command name is wrong."],
        keywords: &["not found", "path", "missing binary"],
    },
    ExitCodeEntry {
        code: 128,
        platform: "unix",
        meaning: "Invalid exit argument or fatal shell condition.",
        notes: &["Git also uses 128 for fatal repository-level errors."],
        keywords: &["fatal", "git", "invalid exit"],
    },
    ExitCodeEntry {
        code: 130,
        platform: "unix",
        meaning: "Script terminated by Ctrl+C (SIGINT).",
        notes: &["128 + 2 where signal 2 is SIGINT."],
        keywords: &["ctrl+c", "sigint", "interrupt"],
    },
    ExitCodeEntry {
        code: 137,
        platform: "unix/linux",
        meaning: "Process killed by SIGKILL.",
        notes: &["Common in containers when the kernel OOM killer terminates a process."],
        keywords: &["oom", "killed", "sigkill", "memory"],
    },
    ExitCodeEntry {
        code: 143,
        platform: "unix/linux",
        meaning: "Process terminated by SIGTERM.",
        notes: &["128 + 15 where signal 15 is SIGTERM."],
        keywords: &["sigterm", "terminated", "graceful stop"],
    },
    ExitCodeEntry {
        code: 255,
        platform: "unix",
        meaning: "Exit status out of range or fatal application failure.",
        notes: &["SSH also commonly returns 255 for connection-level failures."],
        keywords: &["ssh", "fatal", "range"],
    },
    ExitCodeEntry {
        code: 9009,
        platform: "windows",
        meaning: "Command not recognized by cmd.exe.",
        notes: &["Often means the executable is missing from PATH or the command name is misspelled."],
        keywords: &["windows", "not recognized", "cmd", "path"],
    },
];

pub fn shell_cheat_sheet(input: &str) -> Result<CommandExecutionResult, AppError> {
    let query = input.trim();
    if query.is_empty() {
        return Ok(info(
            "shell.cheatsheet",
            "Shell Cheat Sheet Bank",
            overview_output(),
            "Search tar, find, sed, awk, or chmod by tool, flag, or task.",
        ));
    }

    let matches = ranked_matches(query);
    if matches.is_empty() {
        return Ok(info(
            "shell.cheatsheet",
            "No Cheat Sheet Matches",
            format!(
                "No entries matched '{}'. Try:\n- tar extract\n- find name\n- sed replace\n- awk columns\n- chmod recursive",
                query
            ),
            "No matching shell references found.",
        ));
    }

    let top_matches = matches.into_iter().take(6).collect::<Vec<_>>();
    Ok(success(
        "shell.cheatsheet",
        "Shell Cheat Sheet Results",
        top_matches
            .iter()
            .enumerate()
            .map(|(index, entry)| format_cheat_sheet_entry(index + 1, entry))
            .collect::<Vec<_>>()
            .join("\n\n"),
        if top_matches.len() == 1 {
            format!("1 match for '{}'.", query)
        } else {
            format!("{} matches for '{}'.", top_matches.len(), query)
        },
    ))
}

pub fn git_reconstruct(input: &str) -> Result<CommandExecutionResult, AppError> {
    let query = input.trim();
    if query.is_empty() {
        let output = GIT_BLUEPRINTS
            .iter()
            .enumerate()
            .map(|(index, entry)| format!("{}. {} [{}]\n{}", index + 1, entry.title, entry.key, entry.summary))
            .collect::<Vec<_>>()
            .join("\n\n");
        return Ok(info(
            "shell.git-wizard",
            "Git Reconstruction Wizard",
            format!("Available blueprints:\n\n{}\n\nSearch with terms like:\n- interactive rebase\n- squash commits\n- undo keep changes\n- hard reset remote", output),
            "Search Git workflows to get command blueprints.",
        ));
    }

    let mut scored = GIT_BLUEPRINTS
        .iter()
        .filter_map(|entry| {
            let score = score_terms(query, &[entry.title, entry.summary, entry.key, &entry.keywords.join(" ")]);
            (score > 0).then_some((score, entry))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.title.cmp(right.1.title)));

    if scored.is_empty() {
        return Ok(info(
            "shell.git-wizard",
            "No Git Blueprint Matches",
            format!("No git blueprint matched '{}'. Try 'rebase', 'squash', 'fixup', 'detached', 'reset', or 'rename branch'.", query),
            "No Git workflow matches found.",
        ));
    }

    let output = scored
        .iter()
        .take(4)
        .enumerate()
        .map(|(index, (_, entry))| format_git_blueprint(index + 1, entry))
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(success(
        "shell.git-wizard",
        "Git Reconstruction Blueprints",
        output,
        format!("{} blueprint match(es).", scored.len().min(4)),
    ))
}

pub fn code_vault(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        let vault = CODE_VAULT.lock().unwrap();
        let mut output = String::from(
            "Commands:\nadd:<name>:<language>\n<code>\n\nget:<name>\nlist\nsearch:<term>\ndel:<name>\nclear\n\nThe vault stores code snippets and returns fenced blocks for syntax-aware rendering.\n",
        );
        if !vault.is_empty() {
            output.push_str("\nStored snippets:\n");
            for (index, snippet) in vault.iter().enumerate().take(10) {
                output.push_str(&format!(
                    "{}. {} [{}] ({} lines)\n",
                    index + 1,
                    snippet.name,
                    snippet.language,
                    snippet.content.lines().count().max(1)
                ));
            }
        }
        return Ok(info(
            "shell.code-vault",
            "Code Vault",
            output,
            format!("{} snippet(s) stored.", vault.len()),
        ));
    }

    if let Some(rest) = trimmed.strip_prefix("add:") {
        let mut lines = rest.lines();
        let header = lines.next().unwrap_or_default();
        let content = lines.collect::<Vec<_>>().join("\n").trim().to_string();
        let mut parts = header.splitn(2, ':');
        let name = parts.next().unwrap_or_default().trim();
        let language = parts.next().unwrap_or("text").trim();
        if name.is_empty() || content.is_empty() {
            return Ok(info(
                "shell.code-vault",
                "Code Vault Add",
                "Format:\nadd:<name>:<language>\n<code>".into(),
                "Snippet name and code are required.",
            ));
        }

        let mut vault = CODE_VAULT.lock().unwrap();
        if let Some(existing) = vault.iter_mut().find(|item| item.name.eq_ignore_ascii_case(name)) {
            existing.language = language.to_string();
            existing.content = content.clone();
        } else {
            if vault.len() >= MAX_VAULT_ITEMS {
                vault.pop_front();
            }
            vault.push_back(CodeSnippet {
                name: name.to_string(),
                language: language.to_string(),
                content: content.clone(),
            });
        }

        return Ok(success(
            "shell.code-vault",
            "Snippet Saved",
            fenced_code(language, &content),
            format!("Saved '{}' as {}.", name, language),
        ));
    }

    if trimmed.eq_ignore_ascii_case("list") {
        let vault = CODE_VAULT.lock().unwrap();
        if vault.is_empty() {
            return Ok(info(
                "shell.code-vault",
                "Code Vault",
                "No snippets stored.".into(),
                "Vault is empty.",
            ));
        }
        let output = vault
            .iter()
            .enumerate()
            .map(|(index, snippet)| {
                format!(
                    "{}. {} [{}] ({} lines)",
                    index + 1,
                    snippet.name,
                    snippet.language,
                    snippet.content.lines().count().max(1)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(success(
            "shell.code-vault",
            "Code Vault Snippets",
            output,
            format!("{} snippet(s).", vault.len()),
        ));
    }

    if let Some(name) = trimmed.strip_prefix("get:") {
        let name = name.trim();
        let vault = CODE_VAULT.lock().unwrap();
        if let Some(snippet) = vault.iter().find(|item| item.name.eq_ignore_ascii_case(name)) {
            return Ok(success(
                "shell.code-vault",
                &format!("Snippet: {}", snippet.name),
                fenced_code(&snippet.language, &snippet.content),
                format!("Returned '{}' snippet.", snippet.name),
            ));
        }
        return Ok(info(
            "shell.code-vault",
            "Snippet Not Found",
            format!("No snippet named '{}'.", name),
            "Snippet not found.",
        ));
    }

    if let Some(query) = trimmed.strip_prefix("search:") {
        let query = query.trim().to_lowercase();
        let vault = CODE_VAULT.lock().unwrap();
        let matches = vault
            .iter()
            .filter(|snippet| {
                snippet.name.to_lowercase().contains(&query)
                    || snippet.language.to_lowercase().contains(&query)
                    || snippet.content.to_lowercase().contains(&query)
            })
            .collect::<Vec<_>>();
        if matches.is_empty() {
            return Ok(info(
                "shell.code-vault",
                "Code Vault Search",
                format!("No snippets matched '{}'.", query),
                "No snippet matches found.",
            ));
        }
        let output = matches
            .iter()
            .enumerate()
            .map(|(index, snippet)| format!("{}. {} [{}]", index + 1, snippet.name, snippet.language))
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(success(
            "shell.code-vault",
            "Code Vault Search",
            output,
            format!("{} snippet match(es).", matches.len()),
        ));
    }

    if let Some(name) = trimmed.strip_prefix("del:") {
        let name = name.trim();
        let mut vault = CODE_VAULT.lock().unwrap();
        let before = vault.len();
        vault.retain(|snippet| !snippet.name.eq_ignore_ascii_case(name));
        return Ok(if before == vault.len() {
            info(
                "shell.code-vault",
                "Snippet Not Found",
                format!("No snippet named '{}'.", name),
                "Snippet not found.",
            )
        } else {
            success(
                "shell.code-vault",
                "Snippet Deleted",
                format!("Deleted '{}'.", name),
                format!("{} snippet(s) remain.", vault.len()),
            )
        });
    }

    if trimmed.eq_ignore_ascii_case("clear") {
        let mut vault = CODE_VAULT.lock().unwrap();
        let count = vault.len();
        vault.clear();
        return Ok(success(
            "shell.code-vault",
            "Code Vault Cleared",
            format!("Removed {} snippet(s).", count),
            "Vault cleared.",
        ));
    }

    Ok(info(
        "shell.code-vault",
        "Code Vault Usage",
        "Commands:\nadd:<name>:<language>\n<code>\n\nget:<name>\nlist\nsearch:<term>\ndel:<name>\nclear".into(),
        "Unsupported code vault command.",
    ))
}

pub fn shell_history_search(input: &str) -> Result<CommandExecutionResult, AppError> {
    let query = input.trim().to_lowercase();
    if query.is_empty() {
        let sources = history_sources()
            .into_iter()
            .map(|path| format!("- {}", path.display()))
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(info(
            "shell.history",
            "Shell History Search",
            format!("Search local shell history files with a query.\n\nSources checked:\n{}", sources),
            "Provide a search term to scan shell history.",
        ));
    }

    let mut matches = Vec::new();
    for source in history_sources().into_iter().filter(|path| path.exists()) {
        if let Ok(contents) = fs::read_to_string(&source) {
            for (index, line) in contents.lines().enumerate() {
                let normalized = normalize_history_line(line);
                if normalized.to_lowercase().contains(&query) {
                    matches.push((source.clone(), index + 1, normalized));
                }
            }
        }
    }

    if matches.is_empty() {
        return Ok(info(
            "shell.history",
            "Shell History Search",
            format!("No history entries matched '{}'.", query),
            "No shell history matches found.",
        ));
    }

    let output = matches
        .iter()
        .take(20)
        .enumerate()
        .map(|(index, (path, line_no, command))| {
            format!("{}. {}:{}\n   {}", index + 1, path.display(), line_no, command)
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(success(
        "shell.history",
        "Shell History Matches",
        output,
        format!("{} history match(es).", matches.len()),
    ))
}

pub fn alias_blueprint(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "shell.alias",
            "Alias Blueprint Generator",
            "Formats:\nname:<alias>\ncommand:<full command>\nshell:<bash|zsh|fish|powershell|cmd|all>\n\nOr one-line shortcut:\n<alias> => <command>".into(),
            "Provide an alias name and command.",
        ));
    }

    let mut alias_name = String::new();
    let mut command = String::new();
    let mut shell = String::from("all");

    if let Some((left, right)) = trimmed.split_once("=>") {
        alias_name = left.trim().to_string();
        command = right.trim().to_string();
    } else {
        for line in trimmed.lines() {
            if let Some(value) = line.strip_prefix("name:") {
                alias_name = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("command:") {
                command = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("shell:") {
                shell = value.trim().to_lowercase();
            }
        }
    }

    if alias_name.is_empty() || command.is_empty() {
        return Ok(info(
            "shell.alias",
            "Alias Blueprint Generator",
            "Both alias name and command are required.\nExample:\nname: gs\ncommand: git status -sb\nshell: all".into(),
            "Missing alias name or command.",
        ));
    }

    let output = build_alias_output(&alias_name, &command, &shell);
    Ok(success(
        "shell.alias",
        "Alias Blueprints",
        output,
        format!("Alias '{}' generated for {}.", alias_name, shell),
    ))
}

pub fn path_translate(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "shell.path-translate",
            "Path Translator",
            "Paste a Windows, POSIX, or WSL-style path.\nExamples:\nC:\\Users\\katwa\\project\n/mnt/c/Users/katwa/project\n/c/Users/katwa/project\n./src/lib".into(),
            "Provide a path to translate.",
        ));
    }

    let translations = translate_path_variants(trimmed);
    Ok(success(
        "shell.path-translate",
        "Path Translation",
        translations.join("\n"),
        "Generated cross-platform path variants.",
    ))
}

pub fn cron_explain(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "shell.cron",
            "Cron Deconstructor",
            "Paste a 5-field cron expression like:\n*/15 9-17 * * 1-5\n0 0 * * *\n@daily".into(),
            "Provide a cron expression to explain.",
        ));
    }

    if let Some(expansion) = expand_cron_shortcut(trimmed) {
        return Ok(success(
            "shell.cron",
            "Cron Shortcut",
            format!("Shortcut: {}\nExpanded: {}\n{}", trimmed, expansion, describe_cron_expression(expansion)?),
            "Expanded cron shortcut.",
        ));
    }

    let description = describe_cron_expression(trimmed)?;
    Ok(success(
        "shell.cron",
        "Cron Explanation",
        description,
        "Cron expression parsed.",
    ))
}

pub fn docker_compose_generate(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "shell.compose",
            "Docker Compose Canvas",
            "Template:\nservice:web\nimage=node:20\nports=3000:3000,9229:9229\nenv=NODE_ENV=development;PORT=3000\nvolumes=.:/app\ncommand=npm run dev\n\nservice:db\nimage=postgres:16\nports=5432:5432\nenv=POSTGRES_PASSWORD=postgres;POSTGRES_DB=app".into(),
            "Provide service blocks to generate compose YAML.",
        ));
    }

    let services = parse_compose_services(trimmed)?;
    let yaml = render_compose_yaml(&services);
    Ok(success(
        "shell.compose",
        "Docker Compose YAML",
        yaml,
        format!("Generated compose file for {} service(s).", services.len()),
    ))
}

pub fn strip_ansi(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input;
    if trimmed.trim().is_empty() {
        return Ok(info(
            "shell.ansi-strip",
            "ANSI Escape Stripper",
            "Paste terminal output containing colors or cursor-control sequences.".into(),
            "Provide ANSI-colored text to clean.",
        ));
    }

    let without_osc = ANSI_OSC_REGEX.replace_all(trimmed, "");
    let cleaned = ANSI_REGEX.replace_all(&without_osc, "").replace('\u{1b}', "");
    Ok(success(
        "shell.ansi-strip",
        "ANSI Stripped Output",
        cleaned,
        "Removed ANSI escape sequences.",
    ))
}

pub fn exit_code_reference(input: &str) -> Result<CommandExecutionResult, AppError> {
    let query = input.trim().to_lowercase();
    if query.is_empty() {
        let output = EXIT_CODES
            .iter()
            .map(|entry| format!("{} [{}] - {}", entry.code, entry.platform, entry.meaning))
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(info(
            "shell.exit-code",
            "Exit Code Reference",
            format!("Common codes:\n{}\n\nSearch by number or keyword like 'oom', 'not found', or 'ctrl+c'.", output),
            "Provide an exit code or search term.",
        ));
    }

    let matches = if let Ok(code) = query.parse::<i32>() {
        EXIT_CODES
            .iter()
            .filter(|entry| entry.code == code)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        let mut entries = EXIT_CODES
            .iter()
            .filter_map(|entry| {
                let score = score_terms(
                    &query,
                    &[entry.meaning, entry.platform, &entry.keywords.join(" "), &entry.notes.join(" ")],
                );
                (score > 0).then_some((score, entry.clone()))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.code.cmp(&right.1.code)));
        entries.into_iter().map(|(_, entry)| entry).collect::<Vec<_>>()
    };

    if matches.is_empty() {
        return Ok(info(
            "shell.exit-code",
            "Exit Code Reference",
            format!("No exit codes matched '{}'.", query),
            "No exit code matches found.",
        ));
    }

    let output = matches
        .iter()
        .map(|entry| {
            format!(
                "{} [{}]\n{}\nNotes:\n{}",
                entry.code,
                entry.platform,
                entry.meaning,
                entry.notes.iter().map(|note| format!("- {}", note)).collect::<Vec<_>>().join("\n")
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(success(
        "shell.exit-code",
        "Exit Code Reference",
        output,
        format!("{} exit code match(es).", matches.len()),
    ))
}

fn overview_output() -> String {
    let tools = ["tar", "find", "sed", "awk", "chmod"].join(", ");
    format!(
        "Supported tools: {tools}\n\nSearch by tool, action, or flag.\n\nExamples:\n- tar extract\n- find '*.rs'\n- sed replace\n- awk csv columns\n- chmod 755\n\nTip: results include syntax, common flags, and copyable examples."
    )
}

fn ranked_matches(query: &str) -> Vec<CheatSheetEntry> {
    let normalized_query = query.to_lowercase();
    let query_terms = normalized_query.split_whitespace().filter(|term| !term.is_empty()).collect::<Vec<_>>();

    let mut scored = CHEAT_SHEET
        .iter()
        .filter_map(|entry| {
            let haystack = searchable_text(entry);
            let mut score = 0i32;

            if entry.tool == normalized_query {
                score += 250;
            }
            if entry.topic.to_lowercase() == normalized_query {
                score += 180;
            }
            if haystack.contains(&normalized_query) {
                score += 120;
            }

            for term in &query_terms {
                if entry.tool == *term {
                    score += 90;
                }
                if entry.topic.to_lowercase().contains(term) {
                    score += 60;
                }
                if entry.summary.to_lowercase().contains(term) {
                    score += 35;
                }
                if entry.syntax.to_lowercase().contains(term) {
                    score += 25;
                }
                if entry.flags.iter().any(|flag| flag.to_lowercase().contains(term)) {
                    score += 22;
                }
                if entry.keywords.iter().any(|keyword| keyword.to_lowercase().contains(term)) {
                    score += 45;
                }
                if entry.examples.iter().any(|example| example.to_lowercase().contains(term)) {
                    score += 18;
                }
            }

            (score > 0).then_some((score, entry.clone()))
        })
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.tool.cmp(right.1.tool))
            .then_with(|| left.1.topic.cmp(right.1.topic))
    });

    scored.into_iter().map(|(_, entry)| entry).collect()
}

fn searchable_text(entry: &CheatSheetEntry) -> String {
    [entry.tool, entry.topic, entry.summary, entry.syntax, &entry.flags.join(" "), &entry.examples.join(" "), &entry.keywords.join(" ")]
        .join(" ")
        .to_lowercase()
}

fn format_cheat_sheet_entry(index: usize, entry: &CheatSheetEntry) -> String {
    let flags = entry.flags.iter().map(|flag| format!("  - {}", flag)).collect::<Vec<_>>().join("\n");
    let examples = entry.examples.iter().map(|example| format!("  {}", example)).collect::<Vec<_>>().join("\n");
    format!(
        "{index}. {tool} | {topic}\n{summary}\nSyntax: {syntax}\nFlags:\n{flags}\nExamples:\n{examples}",
        tool = entry.tool,
        topic = entry.topic,
        summary = entry.summary,
        syntax = entry.syntax,
    )
}

fn format_git_blueprint(index: usize, entry: &BlueprintEntry) -> String {
    let commands = entry.commands.iter().map(|cmd| format!("  {}", cmd)).collect::<Vec<_>>().join("\n");
    let notes = entry.notes.iter().map(|note| format!("  - {}", note)).collect::<Vec<_>>().join("\n");
    format!(
        "{index}. {title} [{key}]\n{summary}\nCommands:\n{commands}\nNotes:\n{notes}",
        title = entry.title,
        key = entry.key,
        summary = entry.summary,
    )
}

fn build_alias_output(alias_name: &str, command: &str, shell: &str) -> String {
    let shell = shell.to_lowercase();
    let escaped_single = command.replace('\'', "'\"'\"'");
    let escaped_double = command.replace('"', "`\"");
    let escaped_cmd = command.replace('"', "^\"");
    let entries = [
        (
            "bash",
            format!("alias {}='{}'", alias_name, escaped_single),
        ),
        (
            "zsh",
            format!("alias {}='{}'", alias_name, escaped_single),
        ),
        (
            "fish",
            format!("alias {} '{}'", alias_name, command),
        ),
        (
            "powershell",
            format!("function {} {{ {} }}", alias_name, escaped_double),
        ),
        (
            "cmd",
            format!("doskey {}={}", alias_name, escaped_cmd),
        ),
    ];

    entries
        .iter()
        .filter(|(target, _)| shell == "all" || shell == *target)
        .map(|(target, blueprint)| format!("{}:\n{}", target, blueprint))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn translate_path_variants(input: &str) -> Vec<String> {
    let normalized = input.trim().replace('\\', "/");
    let mut lines = vec![format!("Input: {}", input)];

    if let Some((drive, rest)) = windows_drive_parts(input) {
        let rest = rest.replace('\\', "/").trim_start_matches('/').to_string();
        lines.push(format!("Windows: {}:\\{}", drive, rest.replace('/', "\\")));
        lines.push(format!("POSIX-ish: /{}/{}", drive.to_ascii_lowercase(), rest));
        lines.push(format!("WSL: /mnt/{}/{}", drive.to_ascii_lowercase(), rest));
        return lines;
    }

    if let Some(stripped) = normalized.strip_prefix("/mnt/") {
        let mut parts = stripped.splitn(2, '/');
        if let (Some(drive), Some(rest)) = (parts.next(), parts.next()) {
            lines.push(format!("Windows: {}:\\{}", drive.to_ascii_uppercase(), rest.replace('/', "\\")));
            lines.push(format!("POSIX-ish: /{}/{}", drive.to_ascii_lowercase(), rest));
            lines.push(format!("WSL: /mnt/{}/{}", drive.to_ascii_lowercase(), rest));
            return lines;
        }
    }

    if normalized.len() > 3 && normalized.starts_with('/') {
        let bytes = normalized.as_bytes();
        if bytes.get(2) == Some(&b'/') && bytes[1].is_ascii_alphabetic() {
            let drive = normalized.chars().nth(1).unwrap_or('c').to_ascii_uppercase();
            let rest = &normalized[3..];
            lines.push(format!("Windows: {}:\\{}", drive, rest.replace('/', "\\")));
            lines.push(format!("POSIX-ish: /{}/{}", drive.to_ascii_lowercase(), rest));
            lines.push(format!("WSL: /mnt/{}/{}", drive.to_ascii_lowercase(), rest));
            return lines;
        }
    }

    lines.push(format!("POSIX: {}", normalized));
    lines.push(format!("Windows-ish: {}", normalized.replace('/', "\\")));
    lines
}

fn windows_drive_parts(input: &str) -> Option<(char, String)> {
    let chars = input.chars().collect::<Vec<_>>();
    if chars.len() >= 3 && chars[1] == ':' && (chars[2] == '\\' || chars[2] == '/') && chars[0].is_ascii_alphabetic() {
        Some((chars[0].to_ascii_uppercase(), input[3..].to_string()))
    } else {
        None
    }
}

fn expand_cron_shortcut(input: &str) -> Option<&'static str> {
    match input {
        "@yearly" | "@annually" => Some("0 0 1 1 *"),
        "@monthly" => Some("0 0 1 * *"),
        "@weekly" => Some("0 0 * * 0"),
        "@daily" | "@midnight" => Some("0 0 * * *"),
        "@hourly" => Some("0 * * * *"),
        _ => None,
    }
}

fn describe_cron_expression(input: &str) -> Result<String, AppError> {
    let fields = input.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 5 {
        return Err(AppError::Internal(
            "cron format must contain exactly 5 fields: minute hour day-of-month month day-of-week".into(),
        ));
    }

    let labels = ["minute", "hour", "day-of-month", "month", "day-of-week"];
    let descriptions = fields
        .iter()
        .zip(labels.iter())
        .map(|(field, label)| format!("{}: {}", label, describe_cron_field(field, label)))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!("Expression: {}\n{}\n\nSummary:\n{}", input, descriptions, summarize_cron(fields)))
}

fn describe_cron_field(field: &str, label: &str) -> String {
    if field == "*" {
        return format!("every {}", label);
    }
    if let Some(rest) = field.strip_prefix("*/") {
        return format!("every {} {}", rest, label);
    }
    if field.contains(',') {
        return format!("specific {} values [{}]", label, field);
    }
    if field.contains('-') {
        return format!("range [{}]", field);
    }
    if let Some((left, right)) = field.split_once('/') {
        return format!("{} every {} steps", left, right);
    }
    format!("exact value [{}]", field)
}

fn summarize_cron(fields: Vec<&str>) -> String {
    format!(
        "Runs at minute {}, hour {}, day-of-month {}, month {}, day-of-week {}.",
        fields[0], fields[1], fields[2], fields[3], fields[4]
    )
}

#[derive(Default)]
struct ComposeService {
    name: String,
    image: Option<String>,
    build: Option<String>,
    command: Option<String>,
    ports: Vec<String>,
    environment: Vec<(String, String)>,
    volumes: Vec<String>,
}

fn parse_compose_services(input: &str) -> Result<Vec<ComposeService>, AppError> {
    let mut services = Vec::new();
    let mut current = ComposeService::default();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(name) = line.strip_prefix("service:") {
            if !current.name.is_empty() {
                services.push(current);
                current = ComposeService::default();
            }
            current.name = name.trim().to_string();
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| AppError::Internal(format!("invalid compose line: {}", line)))?;
        match key.trim() {
            "image" => current.image = Some(value.trim().to_string()),
            "build" => current.build = Some(value.trim().to_string()),
            "command" => current.command = Some(value.trim().to_string()),
            "ports" => {
                current.ports = value
                    .split(',')
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect();
            }
            "env" => {
                current.environment = value
                    .split(';')
                    .filter_map(|pair| pair.split_once('='))
                    .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                    .collect();
            }
            "volumes" => {
                current.volumes = value
                    .split(',')
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect();
            }
            other => {
                return Err(AppError::Internal(format!("unsupported compose key '{}'", other)));
            }
        }
    }

    if !current.name.is_empty() {
        services.push(current);
    }

    if services.is_empty() {
        return Err(AppError::Internal("at least one service:<name> block is required".into()));
    }

    Ok(services)
}

fn render_compose_yaml(services: &[ComposeService]) -> String {
    let mut output = String::from("services:\n");
    for service in services {
        output.push_str(&format!("  {}:\n", service.name));
        if let Some(image) = &service.image {
            output.push_str(&format!("    image: {}\n", image));
        }
        if let Some(build) = &service.build {
            output.push_str(&format!("    build: {}\n", build));
        }
        if let Some(command) = &service.command {
            output.push_str(&format!("    command: {}\n", command));
        }
        if !service.ports.is_empty() {
            output.push_str("    ports:\n");
            for port in &service.ports {
                output.push_str(&format!("      - \"{}\"\n", port));
            }
        }
        if !service.environment.is_empty() {
            output.push_str("    environment:\n");
            for (key, value) in &service.environment {
                output.push_str(&format!("      {}: \"{}\"\n", key, value));
            }
        }
        if !service.volumes.is_empty() {
            output.push_str("    volumes:\n");
            for volume in &service.volumes {
                output.push_str(&format!("      - {}\n", volume));
            }
        }
    }
    output
}

fn history_sources() -> Vec<PathBuf> {
    let mut sources = Vec::new();

    if let Some(home) = user_home_dir() {
        sources.push(home.join(".bash_history"));
        sources.push(home.join(".zsh_history"));
        sources.push(home.join(".ash_history"));
    }

    if let Some(appdata) = env::var_os("APPDATA") {
        sources.push(
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("PowerShell")
                .join("PSReadLine")
                .join("ConsoleHost_history.txt"),
        );
    }

    sources.sort();
    sources.dedup();
    sources
}

fn user_home_dir() -> Option<PathBuf> {
    env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(PathBuf::from))
}

fn normalize_history_line(line: &str) -> String {
    if let Some(stripped) = line.strip_prefix(": ") {
        if let Some((_, command)) = stripped.split_once(';') {
            return command.trim().to_string();
        }
    }
    line.trim().to_string()
}

fn score_terms(query: &str, fields: &[&str]) -> i32 {
    let lowered_query = query.to_lowercase();
    let terms = lowered_query.split_whitespace().collect::<Vec<_>>();
    let haystack = fields.join(" ").to_lowercase();
    let mut score = 0;

    if haystack.contains(&lowered_query) {
        score += 40;
    }
    for term in terms {
        if haystack.contains(term) {
            score += 25;
        }
    }
    score
}

fn fenced_code(language: &str, content: &str) -> String {
    format!("```{}\n{}\n```", language, content)
}

fn info(command_id: &str, title: &str, output: String, summary: impl Into<String>) -> CommandExecutionResult {
    CommandExecutionResult {
        command_id: command_id.into(),
        title: title.into(),
        output,
        status: CommandExecutionStatus::Info,
        summary: summary.into(),
    }
}

fn success(command_id: &str, title: &str, output: String, summary: impl Into<String>) -> CommandExecutionResult {
    CommandExecutionResult {
        command_id: command_id.into(),
        title: title.into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: summary.into(),
    }
}
