use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, RwLock},
    thread,
    time::{Duration, SystemTime},
};

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::{
    error::AppError,
    models::{CommandAction, CommandExecutionResult, CommandExecutionStatus, ScriptExtensionSummary},
};

#[derive(Clone)]
pub struct LoadedScriptExtension {
    pub command: CommandAction,
    pub source_path: PathBuf,
    pub command_path: String,
    pub args: Vec<String>,
    pub working_directory: Option<String>,
}

pub struct ExtensionLoader {
    directory: PathBuf,
    extensions: Arc<RwLock<Vec<LoadedScriptExtension>>>,
    last_scan: Arc<RwLock<Option<SystemTime>>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScriptExtensionManifest {
    id: String,
    title: String,
    subtitle: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default = "default_accepts_input")]
    accepts_input: bool,
    working_directory: Option<String>,
}

impl ExtensionLoader {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let directory = app_handle
            .path()
            .app_data_dir()
            .map_err(|error| AppError::Internal(format!("failed to resolve app data dir: {error}")))?
            .join("extensions");

        fs::create_dir_all(&directory).map_err(|error| {
            AppError::Internal(format!("failed to create extensions directory: {error}"))
        })?;

        let loader = Self {
            directory,
            extensions: Arc::new(RwLock::new(Vec::new())),
            last_scan: Arc::new(RwLock::new(None)),
        };
        loader.refresh()?;
        Ok(loader)
    }

    pub fn start_watcher(&self) {
        let directory = self.directory.clone();
        let extensions = Arc::clone(&self.extensions);
        let last_scan = Arc::clone(&self.last_scan);

        thread::spawn(move || loop {
            let should_refresh = latest_manifest_mtime(&directory)
                .ok()
                .flatten()
                .map(|latest| {
                    let recorded = last_scan.read().ok().and_then(|value| *value);
                    match recorded {
                        Some(previous) => latest > previous,
                        None => true,
                    }
                })
                .unwrap_or(false);

            if should_refresh {
                if let Ok(next) = load_extensions_from_directory(&directory) {
                    if let Ok(mut guard) = extensions.write() {
                        *guard = next;
                    }
                    if let Ok(mut guard) = last_scan.write() {
                        *guard = latest_manifest_mtime(&directory).ok().flatten();
                    }
                }
            }

            thread::sleep(Duration::from_secs(2));
        });
    }

    pub fn refresh(&self) -> Result<(), AppError> {
        let next = load_extensions_from_directory(&self.directory)?;
        *self
            .extensions
            .write()
            .map_err(|_| AppError::Internal("failed to lock extensions".into()))? = next;
        *self
            .last_scan
            .write()
            .map_err(|_| AppError::Internal("failed to lock extensions".into()))? =
            latest_manifest_mtime(&self.directory)?;
        Ok(())
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn command_actions(&self) -> Vec<CommandAction> {
        self.extensions
            .read()
            .map(|guard| guard.iter().map(|item| item.command.clone()).collect())
            .unwrap_or_default()
    }

    pub fn summaries(&self) -> Vec<ScriptExtensionSummary> {
        self.extensions
            .read()
            .map(|guard| {
                guard
                    .iter()
                    .map(|item| ScriptExtensionSummary {
                        id: item.command.id.clone(),
                        title: item.command.title.clone(),
                        subtitle: item.command.subtitle.clone(),
                        source_path: item.source_path.display().to_string(),
                        command_path: item.command_path.clone(),
                        accepts_input: item.command.accepts_input,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn has_command(&self, command_id: &str) -> bool {
        self.extensions
            .read()
            .map(|guard| guard.iter().any(|item| item.command.id == command_id))
            .unwrap_or(false)
    }

    pub fn execute(
        &self,
        command_id: &str,
        input: &str,
    ) -> Result<CommandExecutionResult, AppError> {
        let extension = self
            .extensions
            .read()
            .map_err(|_| AppError::Internal("failed to lock extensions".into()))?
            .iter()
            .find(|item| item.command.id == command_id)
            .cloned()
            .ok_or_else(|| AppError::Internal(format!("unknown extension command id: {command_id}")))?;

        let mut process = Command::new(&extension.command_path);
        process.args(&extension.args);

        if let Some(directory) = &extension.working_directory {
            process.current_dir(directory);
        }

        process.stdout(Stdio::piped()).stderr(Stdio::piped());
        if extension.command.accepts_input {
            process.stdin(Stdio::piped());
        }

        let mut child = process.spawn().map_err(|error| {
            AppError::Internal(format!(
                "failed to launch extension command {}: {error}",
                extension.command.id
            ))
        })?;

        if extension.command.accepts_input {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(input.as_bytes()).map_err(|error| {
                    AppError::Internal(format!(
                        "failed to send input to extension command {}: {error}",
                        extension.command.id
                    ))
                })?;
            }
        }

        let output = child.wait_with_output().map_err(|error| {
            AppError::Internal(format!(
                "failed to read extension command {} output: {error}",
                extension.command.id
            ))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let success = output.status.success();
        let body = if success {
            stdout
        } else if stderr.is_empty() {
            stdout
        } else {
            stderr
        };

        Ok(CommandExecutionResult {
            command_id: extension.command.id,
            title: extension.command.title,
            output: body,
            status: if success {
                CommandExecutionStatus::Success
            } else {
                CommandExecutionStatus::Error
            },
            summary: if success {
                "Extension command completed successfully.".into()
            } else {
                format!("Extension command exited with status {}.", output.status)
            },
        })
    }
}

fn load_extensions_from_directory(directory: &Path) -> Result<Vec<LoadedScriptExtension>, AppError> {
    let mut extensions = Vec::new();
    for entry in fs::read_dir(directory)
        .map_err(|error| AppError::Internal(format!("failed to read extensions directory: {error}")))?
    {
        let entry = entry
            .map_err(|error| AppError::Internal(format!("failed to inspect extension entry: {error}")))?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let contents = fs::read_to_string(&path).map_err(|error| {
            AppError::Internal(format!("failed to read extension manifest {}: {error}", path.display()))
        })?;
        let manifest: ScriptExtensionManifest = serde_json::from_str(&contents).map_err(|error| {
            AppError::Internal(format!(
                "failed to parse extension manifest {}: {error}",
                path.display()
            ))
        })?;

        let mut tags = manifest.tags.clone();
        tags.push("extension".into());
        tags.sort();
        tags.dedup();

        extensions.push(LoadedScriptExtension {
            command: CommandAction {
                id: manifest.id,
                title: manifest.title,
                subtitle: manifest.subtitle,
                category: crate::models::CommandCategory::System,
                tags,
                shortcut: None,
                accepts_input: manifest.accepts_input,
            },
            source_path: path,
            command_path: manifest.command,
            args: manifest.args,
            working_directory: manifest.working_directory,
        });
    }

    extensions.sort_by(|left, right| left.command.title.cmp(&right.command.title));
    Ok(extensions)
}

fn latest_manifest_mtime(directory: &Path) -> Result<Option<SystemTime>, AppError> {
    let mut latest = None;
    for entry in fs::read_dir(directory)
        .map_err(|error| AppError::Internal(format!("failed to read extensions directory: {error}")))?
    {
        let entry = entry
            .map_err(|error| AppError::Internal(format!("failed to inspect extension entry: {error}")))?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .map_err(|error| {
                AppError::Internal(format!("failed to inspect extension manifest {}: {error}", path.display()))
            })?;

        latest = Some(match latest {
            Some(current) if current > modified => current,
            _ => modified,
        });
    }
    Ok(latest)
}

fn default_accepts_input() -> bool {
    true
}
