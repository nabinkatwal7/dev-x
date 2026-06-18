use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

#[derive(Clone)]
struct FileSnapshot {
    path: String,
    size: u64,
    modified: u64,
}

lazy_static::lazy_static! {
    static ref WATCH_SNAPSHOTS: Mutex<HashMap<String, HashMap<String, FileSnapshot>>> = Mutex::new(HashMap::new());
}

pub fn env_audit(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "fs.env-audit",
            "Env Audit Matrix",
            "Compare multiple .env payloads or file paths.\n\nFormats:\npath:.env\n---\npath:.env.local\n\nOr paste raw blocks:\nname:base\nKEY=a\n---\nname:prod\nKEY=a\nEXTRA=b".into(),
            "Provide .env files or raw environment blocks to compare.",
        ));
    }

    let blocks = split_blocks(trimmed);
    let mut env_sets = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        let (name, content) = parse_named_or_path_block(block, index + 1)?;
        env_sets.push((name, parse_env_map(&content)));
    }

    let mut all_keys = BTreeMap::new();
    for (_, env) in &env_sets {
        for key in env.keys() {
            all_keys.insert(key.clone(), ());
        }
    }

    let mut output = String::new();
    for key in all_keys.keys() {
        let mut row = vec![format!("{}:", key)];
        for (name, env) in &env_sets {
            row.push(format!(
                "{}={}",
                name,
                env.get(key).map(|value| mask_env_value(key, value)).unwrap_or_else(|| "[missing]".into())
            ));
        }
        output.push_str(&row.join(" | "));
        output.push('\n');
    }

    let mut findings = Vec::new();
    for key in all_keys.keys() {
        let present = env_sets.iter().filter(|(_, env)| env.contains_key(key)).count();
        if present != env_sets.len() {
            findings.push(format!("{} missing in {} profile(s)", key, env_sets.len() - present));
        }
    }

    if findings.is_empty() {
        findings.push("All keys are present across every environment.".into());
    }

    output.push_str("\nFindings:\n");
    for finding in findings {
        output.push_str(&format!("- {}\n", finding));
    }

    Ok(success(
        "fs.env-audit",
        "Env Audit Matrix",
        output,
        format!("Compared {} environment set(s).", env_sets.len()),
    ))
}

pub fn duplicate_scan(input: &str) -> Result<CommandExecutionResult, AppError> {
    let path = resolve_path_or_default(input.trim(), ".");
    let mut files = Vec::new();
    collect_files(&path, &mut files)?;

    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for file in files {
        if let Ok(metadata) = fs::metadata(&file) {
            by_size.entry(metadata.len()).or_default().push(file);
        }
    }

    let mut duplicates = Vec::new();
    for (size, paths) in by_size {
        if paths.len() < 2 {
            continue;
        }
        let mut by_hash: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for path in paths {
            let hash = hash_file(&path)?;
            by_hash.entry(hash).or_default().push(path);
        }
        for (hash, items) in by_hash {
            if items.len() > 1 {
                duplicates.push((size, hash, items));
            }
        }
    }

    if duplicates.is_empty() {
        return Ok(info(
            "fs.duplicate-scan",
            "Duplicate File Scan",
            format!("No duplicates found under {}.", path.display()),
            "No duplicate files found.",
        ));
    }

    duplicates.sort_by(|left, right| right.0.cmp(&left.0));
    let output = duplicates
        .iter()
        .enumerate()
        .map(|(index, (size, hash, items))| {
            format!(
                "{}. {} bytes | sha256:{}\n{}",
                index + 1,
                size,
                &hash[..12],
                items
                    .iter()
                    .map(|path| format!("   - {}", path.display()))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(success(
        "fs.duplicate-scan",
        "Duplicate File Scan",
        output,
        format!("Found {} duplicate group(s).", duplicates.len()),
    ))
}

pub fn symlink_matrix(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("list") {
        let path = PathBuf::from(".");
        let mut links = Vec::new();
        collect_symlinks(&path, &mut links)?;
        if links.is_empty() {
            return Ok(info(
                "fs.symlink-matrix",
                "Symlink Matrix",
                "No symlinks found in the current workspace.\n\nCommands:\nlist\ninspect:<path>\ncreate:<link>|<target>\nremove:<link>".into(),
                "No symlinks discovered.",
            ));
        }
        let output = links
            .iter()
            .enumerate()
            .map(|(index, (link, target))| format!("{}. {} -> {}", index + 1, link.display(), target.display()))
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(success(
            "fs.symlink-matrix",
            "Symlink Matrix",
            output,
            format!("Found {} symlink(s).", links.len()),
        ));
    }

    if let Some(path) = trimmed.strip_prefix("inspect:") {
        let path = PathBuf::from(path.trim());
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| AppError::Internal(format!("failed to inspect {}: {}", path.display(), error)))?;
        if !metadata.file_type().is_symlink() {
            return Ok(info(
                "fs.symlink-matrix",
                "Symlink Inspect",
                format!("{} is not a symlink.", path.display()),
                "Path is not a symlink.",
            ));
        }
        let target = fs::read_link(&path)
            .map_err(|error| AppError::Internal(format!("failed to read link target: {}", error)))?;
        return Ok(success(
            "fs.symlink-matrix",
            "Symlink Inspect",
            format!("Link: {}\nTarget: {}", path.display(), target.display()),
            "Symlink inspected.",
        ));
    }

    if let Some(args) = trimmed.strip_prefix("create:") {
        let (link, target) = args
            .split_once('|')
            .ok_or_else(|| AppError::Internal("format: create:<link>|<target>".into()))?;
        let link = PathBuf::from(link.trim());
        let target = PathBuf::from(target.trim());
        let target_metadata = fs::metadata(&target)
            .map_err(|error| AppError::Internal(format!("failed to stat target {}: {}", target.display(), error)))?;
        #[cfg(target_os = "windows")]
        {
            if target_metadata.is_dir() {
                std::os::windows::fs::symlink_dir(&target, &link)
                    .map_err(|error| AppError::Internal(format!("failed to create symlink: {}", error)))?;
            } else {
                std::os::windows::fs::symlink_file(&target, &link)
                    .map_err(|error| AppError::Internal(format!("failed to create symlink: {}", error)))?;
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::os::unix::fs::symlink(&target, &link)
                .map_err(|error| AppError::Internal(format!("failed to create symlink: {}", error)))?;
        }
        return Ok(success(
            "fs.symlink-matrix",
            "Symlink Created",
            format!("Created {} -> {}", link.display(), target.display()),
            "Symlink created.",
        ));
    }

    if let Some(path) = trimmed.strip_prefix("remove:") {
        let path = PathBuf::from(path.trim());
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| AppError::Internal(format!("failed to inspect {}: {}", path.display(), error)))?;
        if !metadata.file_type().is_symlink() {
            return Ok(info(
                "fs.symlink-matrix",
                "Symlink Remove",
                format!("{} is not a symlink.", path.display()),
                "Path is not a symlink.",
            ));
        }
        if metadata.is_dir() {
            fs::remove_dir(&path)
                .map_err(|error| AppError::Internal(format!("failed to remove symlink dir: {}", error)))?;
        } else {
            fs::remove_file(&path)
                .map_err(|error| AppError::Internal(format!("failed to remove symlink file: {}", error)))?;
        }
        return Ok(success(
            "fs.symlink-matrix",
            "Symlink Removed",
            format!("Removed {}.", path.display()),
            "Symlink removed.",
        ));
    }

    Ok(info(
        "fs.symlink-matrix",
        "Symlink Matrix",
        "Commands:\nlist\ninspect:<path>\ncreate:<link>|<target>\nremove:<link>".into(),
        "Unsupported symlink command.",
    ))
}

pub fn log_tail(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "fs.log-tail",
            "Log Tail",
            "Format:\n<path>\nOptional second line: lines=200".into(),
            "Provide a log file path.",
        ));
    }

    let mut lines_limit = 120usize;
    let mut path = "";
    for (index, line) in trimmed.lines().enumerate() {
        if index == 0 {
            path = line.trim();
        } else if let Some(value) = line.trim().strip_prefix("lines=") {
            lines_limit = value.trim().parse::<usize>().unwrap_or(120).clamp(1, 2000);
        }
    }

    let contents = fs::read_to_string(path)
        .map_err(|error| AppError::Internal(format!("failed to read log {}: {}", path, error)))?;
    let lines = contents.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(lines_limit);
    let output = lines[start..].join("\n");

    Ok(success(
        "fs.log-tail",
        "Log Tail",
        output,
        format!("Showing last {} line(s).", lines.len() - start),
    ))
}

pub fn file_sentinel(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "fs.file-sentinel",
            "File Change Sentinel",
            "Commands:\nwatch:<path>\ndiff:<path>\nscan:<path>\nclear".into(),
            "Use watch to snapshot a directory, then diff it later.",
        ));
    }

    if trimmed.eq_ignore_ascii_case("clear") {
        WATCH_SNAPSHOTS.lock().unwrap().clear();
        return Ok(success(
            "fs.file-sentinel",
            "File Change Sentinel",
            "Cleared all snapshots.".into(),
            "Snapshots cleared.",
        ));
    }

    let (mode, path) = trimmed
        .split_once(':')
        .ok_or_else(|| AppError::Internal("format: watch:<path> or diff:<path>".into()))?;
    let path = resolve_path_or_default(path.trim(), ".");
    let current = snapshot_directory(&path)?;

    match mode.trim() {
        "scan" => {
            let output = current
                .values()
                .take(100)
                .map(|entry| format!("{} | {} bytes | {}", entry.path, entry.size, entry.modified))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(success(
                "fs.file-sentinel",
                "Directory Snapshot",
                output,
                format!("Scanned {} file(s).", current.len()),
            ))
        }
        "watch" => {
            WATCH_SNAPSHOTS
                .lock()
                .unwrap()
                .insert(path.display().to_string(), current);
            Ok(success(
                "fs.file-sentinel",
                "Directory Watch Snapshot",
                format!("Stored snapshot for {}.", path.display()),
                "Snapshot stored.",
            ))
        }
        "diff" => {
            let mut store = WATCH_SNAPSHOTS.lock().unwrap();
            let key = path.display().to_string();
            let previous = store.get(&key).cloned().ok_or_else(|| {
                AppError::Internal(format!("no snapshot stored for {}. Run watch:{} first.", key, key))
            })?;
            let mut output = Vec::new();

            for (path_key, current_entry) in &current {
                match previous.get(path_key) {
                    None => output.push(format!("+ {}", path_key)),
                    Some(old) if old.size != current_entry.size || old.modified != current_entry.modified => {
                        output.push(format!("~ {}", path_key))
                    }
                    _ => {}
                }
            }
            for path_key in previous.keys() {
                if !current.contains_key(path_key) {
                    output.push(format!("- {}", path_key));
                }
            }

            store.insert(key, current);
            if output.is_empty() {
                output.push("No changes detected since the last snapshot.".into());
            }
            Ok(success(
                "fs.file-sentinel",
                "Directory Diff",
                output.join("\n"),
                "Snapshot diff complete.",
            ))
        }
        _ => Ok(info(
            "fs.file-sentinel",
            "File Change Sentinel",
            "Commands:\nwatch:<path>\ndiff:<path>\nscan:<path>\nclear".into(),
            "Unsupported sentinel command.",
        )),
    }
}

pub fn hex_inspector(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "fs.hex-inspect",
            "Hex Inspector",
            "Format:\n<path>\nOptional second line: bytes=256".into(),
            "Provide a file path to inspect.",
        ));
    }

    let mut path = "";
    let mut limit = 256usize;
    for (index, line) in trimmed.lines().enumerate() {
        if index == 0 {
            path = line.trim();
        } else if let Some(value) = line.trim().strip_prefix("bytes=") {
            limit = value.trim().parse::<usize>().unwrap_or(256).clamp(16, 4096);
        }
    }

    let mut file = fs::File::open(path)
        .map_err(|error| AppError::Internal(format!("failed to open {}: {}", path, error)))?;
    let mut buffer = vec![0u8; limit];
    let read = file
        .read(&mut buffer)
        .map_err(|error| AppError::Internal(format!("failed to read {}: {}", path, error)))?;
    buffer.truncate(read);

    let output = buffer
        .chunks(16)
        .enumerate()
        .map(|(index, chunk)| {
            let hex = chunk.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<_>>().join(" ");
            let ascii = chunk
                .iter()
                .map(|byte| if byte.is_ascii_graphic() || *byte == b' ' { *byte as char } else { '.' })
                .collect::<String>();
            format!("{:08X}  {:<47}  {}", index * 16, hex, ascii)
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(success(
        "fs.hex-inspect",
        "Hex Inspector",
        output,
        format!("Read {} byte(s).", read),
    ))
}

pub fn batch_rename(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "fs.batch-rename",
            "Batch Rename Deck",
            "Format:\ntemplate:{index:03}-{stem}{ext}\nfile1\nfile2\nfile3".into(),
            "Provide a rename template and file list.",
        ));
    }

    let mut lines = trimmed.lines();
    let template_line = lines
        .next()
        .ok_or_else(|| AppError::Internal("template line is required".into()))?;
    let template = template_line
        .strip_prefix("template:")
        .ok_or_else(|| AppError::Internal("first line must start with template:".into()))?
        .trim();
    let files = lines
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Ok(info(
            "fs.batch-rename",
            "Batch Rename Deck",
            "No files supplied after the template line.".into(),
            "No files to rename.",
        ));
    }

    let output = files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let path = Path::new(file);
            let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or("item");
            let ext = path.extension().and_then(|value| value.to_str()).map(|ext| format!(".{}", ext)).unwrap_or_default();
            let renamed = apply_rename_template(template, index + 1, stem, &ext);
            format!("{} -> {}", file, renamed)
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(success(
        "fs.batch-rename",
        "Batch Rename Preview",
        output,
        format!("Generated {} rename preview(s).", files.len()),
    ))
}

pub fn disk_explorer(input: &str) -> Result<CommandExecutionResult, AppError> {
    let path = resolve_path_or_default(input.trim(), ".");
    let mut sizes = Vec::new();
    accumulate_sizes(&path, &mut sizes)?;
    sizes.sort_by(|left, right| right.1.cmp(&left.1));
    let output = sizes
        .iter()
        .take(25)
        .enumerate()
        .map(|(index, (path, size))| format!("{}. {} | {}", index + 1, human_size(*size), path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(success(
        "fs.disk-explorer",
        "Disk Allocation Explorer",
        output,
        format!("Ranked {} filesystem nodes.", sizes.len()),
    ))
}

pub fn encoding_convert(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "fs.encoding-convert",
            "Encoding Conversion",
            "Format:\nfrom:utf8\nto:utf16le\n\n<text>\n\nSupported: utf8, utf16le, utf16be, ascii".into(),
            "Provide source and target encodings with text content.",
        ));
    }

    let parts = trimmed.splitn(3, '\n').collect::<Vec<_>>();
    if parts.len() < 3 {
        return Err(AppError::Internal("format requires from:, to:, blank line/text body".into()));
    }
    let from = parts[0].trim().strip_prefix("from:").unwrap_or("utf8").trim().to_lowercase();
    let to = parts[1].trim().strip_prefix("to:").unwrap_or("utf8").trim().to_lowercase();
    let body = parts[2];
    let bytes = encode_text(body, &from)?;
    let output = decode_bytes(&bytes, &to)?;
    Ok(success(
        "fs.encoding-convert",
        "Encoding Conversion",
        output,
        format!("Converted {} -> {} ({} byte(s)).", from, to, bytes.len()),
    ))
}

pub fn archive_integrity(input: &str) -> Result<CommandExecutionResult, AppError> {
    let path = PathBuf::from(input.trim());
    if input.trim().is_empty() {
        return Ok(info(
            "fs.archive-check",
            "Archive Integrity Sandbox",
            "Provide a .zip, .tar, .gz, or .tgz path to inspect headers and archive structure.".into(),
            "Provide an archive path.",
        ));
    }

    let mut file = fs::File::open(&path)
        .map_err(|error| AppError::Internal(format!("failed to open {}: {}", path.display(), error)))?;
    let metadata = file
        .metadata()
        .map_err(|error| AppError::Internal(format!("failed to stat {}: {}", path.display(), error)))?;
    let mut header = [0u8; 512];
    let read = file
        .read(&mut header)
        .map_err(|error| AppError::Internal(format!("failed to read archive header: {}", error)))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| AppError::Internal(format!("failed to seek archive: {}", error)))?;

    let file_type = detect_archive_type(&header[..read]);
    let mut output = format!(
        "Path: {}\nSize: {}\nDetected: {}",
        path.display(),
        human_size(metadata.len()),
        file_type
    );

    if file_type == "tar" && read >= 265 {
        let magic = std::str::from_utf8(&header[257..262]).unwrap_or("").trim_matches('\0');
        output.push_str(&format!("\nUSTAR magic: {}", magic));
    }

    if file_type == "zip" {
        let mut tail = Vec::new();
        file.seek(SeekFrom::End(-64))
            .or_else(|_| file.seek(SeekFrom::Start(0)))
            .map_err(|error| AppError::Internal(format!("failed to seek zip tail: {}", error)))?;
        file.read_to_end(&mut tail)
            .map_err(|error| AppError::Internal(format!("failed to read zip tail: {}", error)))?;
        let valid = tail.windows(4).any(|window| window == [0x50, 0x4B, 0x05, 0x06]);
        output.push_str(&format!("\nCentral directory footer present: {}", valid));
    }

    if file_type == "gzip" {
        output.push_str(&format!(
            "\nCompression method: {}\nFlags: 0x{:02X}",
            header.get(2).copied().unwrap_or_default(),
            header.get(3).copied().unwrap_or_default()
        ));
    }

    Ok(success(
        "fs.archive-check",
        "Archive Integrity Sandbox",
        output,
        "Archive header inspection complete.",
    ))
}

fn split_blocks(input: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in input.lines() {
        if line.trim() == "---" {
            if !current.trim().is_empty() {
                blocks.push(current.trim().to_string());
            }
            current.clear();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    if !current.trim().is_empty() {
        blocks.push(current.trim().to_string());
    }
    blocks
}

fn parse_named_or_path_block(block: &str, index: usize) -> Result<(String, String), AppError> {
    let mut lines = block.lines();
    let first = lines.next().unwrap_or_default().trim();
    if let Some(path) = first.strip_prefix("path:") {
        let path = path.trim();
        let contents = fs::read_to_string(path)
            .map_err(|error| AppError::Internal(format!("failed to read env file {}: {}", path, error)))?;
        return Ok((Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path).into(), contents));
    }
    if let Some(name) = first.strip_prefix("name:") {
        return Ok((name.trim().into(), lines.collect::<Vec<_>>().join("\n")));
    }
    Ok((format!("env-{}", index), block.into()))
}

fn parse_env_map(contents: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().into(), value.trim().into());
        }
    }
    map
}

fn mask_env_value(key: &str, value: &str) -> String {
    let lower = key.to_lowercase();
    if lower.contains("secret") || lower.contains("token") || lower.contains("password") || lower.contains("key") {
        if value.len() <= 6 {
            return "[redacted]".into();
        }
        return format!("{}***{}", &value[..3], &value[value.len() - 2..]);
    }
    value.into()
}

fn resolve_path_or_default(input: &str, default: &str) -> PathBuf {
    if input.is_empty() {
        PathBuf::from(default)
    } else {
        PathBuf::from(input)
    }
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| AppError::Internal(format!("failed to inspect {}: {}", path.display(), error)))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)
            .map_err(|error| AppError::Internal(format!("failed to read dir {}: {}", path.display(), error)))?
        {
            let entry = entry.map_err(|error| AppError::Internal(format!("failed to read directory entry: {}", error)))?;
            collect_files(&entry.path(), files)?;
        }
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String, AppError> {
    let mut file = fs::File::open(path)
        .map_err(|error| AppError::Internal(format!("failed to open {}: {}", path.display(), error)))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| AppError::Internal(format!("failed to read {}: {}", path.display(), error)))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_symlinks(path: &Path, links: &mut Vec<(PathBuf, PathBuf)>) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| AppError::Internal(format!("failed to inspect {}: {}", path.display(), error)))?;
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(path)
            .map_err(|error| AppError::Internal(format!("failed to read symlink {}: {}", path.display(), error)))?;
        links.push((path.to_path_buf(), target));
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)
            .map_err(|error| AppError::Internal(format!("failed to read dir {}: {}", path.display(), error)))?
        {
            let entry = entry.map_err(|error| AppError::Internal(format!("failed to read directory entry: {}", error)))?;
            collect_symlinks(&entry.path(), links)?;
        }
    }
    Ok(())
}

fn snapshot_directory(path: &Path) -> Result<HashMap<String, FileSnapshot>, AppError> {
    let mut files = Vec::new();
    collect_files(path, &mut files)?;
    let mut map = HashMap::new();
    for file in files {
        let metadata = fs::metadata(&file)
            .map_err(|error| AppError::Internal(format!("failed to stat {}: {}", file.display(), error)))?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        let key = file.display().to_string();
        map.insert(
            key.clone(),
            FileSnapshot {
                path: key,
                size: metadata.len(),
                modified,
            },
        );
    }
    Ok(map)
}

fn apply_rename_template(template: &str, index: usize, stem: &str, ext: &str) -> String {
    let mut result = template.replace("{stem}", stem).replace("{ext}", ext);
    if result.contains("{index:03}") {
        result = result.replace("{index:03}", &format!("{:03}", index));
    }
    if result.contains("{index:02}") {
        result = result.replace("{index:02}", &format!("{:02}", index));
    }
    result.replace("{index}", &index.to_string())
}

fn accumulate_sizes(path: &Path, sizes: &mut Vec<(PathBuf, u64)>) -> Result<u64, AppError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| AppError::Internal(format!("failed to inspect {}: {}", path.display(), error)))?;
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }
    if metadata.is_file() {
        let size = metadata.len();
        sizes.push((path.to_path_buf(), size));
        return Ok(size);
    }

    let mut total = 0;
    for entry in fs::read_dir(path)
        .map_err(|error| AppError::Internal(format!("failed to read dir {}: {}", path.display(), error)))?
    {
        let entry = entry.map_err(|error| AppError::Internal(format!("failed to read directory entry: {}", error)))?;
        total += accumulate_sizes(&entry.path(), sizes)?;
    }
    sizes.push((path.to_path_buf(), total));
    Ok(total)
}

fn encode_text(text: &str, encoding: &str) -> Result<Vec<u8>, AppError> {
    match encoding {
        "utf8" | "utf-8" => Ok(text.as_bytes().to_vec()),
        "ascii" => Ok(text
            .chars()
            .map(|char| if char.is_ascii() { char as u8 } else { b'?' })
            .collect()),
        "utf16le" | "utf-16le" => Ok(text
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect()),
        "utf16be" | "utf-16be" => Ok(text
            .encode_utf16()
            .flat_map(|unit| unit.to_be_bytes())
            .collect()),
        _ => Err(AppError::Internal(format!("unsupported source encoding: {}", encoding))),
    }
}

fn decode_bytes(bytes: &[u8], encoding: &str) -> Result<String, AppError> {
    match encoding {
        "utf8" | "utf-8" => String::from_utf8(bytes.to_vec())
            .map_err(|error| AppError::Internal(format!("utf8 decode failed: {}", error))),
        "ascii" => Ok(bytes
            .iter()
            .map(|byte| if byte.is_ascii() { *byte as char } else { '?' })
            .collect()),
        "utf16le" | "utf-16le" => decode_utf16(bytes, true),
        "utf16be" | "utf-16be" => decode_utf16(bytes, false),
        _ => Err(AppError::Internal(format!("unsupported target encoding: {}", encoding))),
    }
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> Result<String, AppError> {
    if bytes.len() % 2 != 0 {
        return Err(AppError::Internal("utf16 byte length must be even".into()));
    }
    let units = bytes
        .chunks(2)
        .map(|chunk| {
            if little_endian {
                u16::from_le_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_be_bytes([chunk[0], chunk[1]])
            }
        })
        .collect::<Vec<_>>();
    String::from_utf16(&units).map_err(|error| AppError::Internal(format!("utf16 decode failed: {}", error)))
}

fn detect_archive_type(header: &[u8]) -> &'static str {
    if header.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        "zip"
    } else if header.starts_with(&[0x1F, 0x8B]) {
        "gzip"
    } else if header.len() >= 262 && &header[257..262] == b"ustar" {
        "tar"
    } else {
        "unknown"
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", value, UNITS[unit])
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
