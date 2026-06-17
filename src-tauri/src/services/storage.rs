use std::{fs, path::PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Manager};

use crate::{
    app,
    error::AppError,
    models::{
        AppHealth, AppSettings, BootstrapPayload, CommandAction, CommandHistoryEntry, CommandUsageEntry,
        ConfigurationSnapshot, PinnedModule, RecordCommandExecutionPayload, SaveWorkspaceProfilePayload,
        ScriptExtensionSummary, SearchHistoryPayload, UpdateSettingsPayload, WorkspaceProfile,
    },
};

pub struct StorageService {
    db_path: PathBuf,
}

impl StorageService {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|error| AppError::Internal(format!("failed to resolve app data dir: {error}")))?;

        fs::create_dir_all(&app_data_dir)
            .map_err(|error| AppError::Internal(format!("failed to create app data dir: {error}")))?;

        let service = Self {
            db_path: app_data_dir.join("devforge.sqlite3"),
        };
        service.initialize()?;
        Ok(service)
    }

    pub fn load_bootstrap_payload(
        &self,
        commands: Vec<CommandAction>,
        tray_ready: bool,
        extension_directory: String,
        extensions: Vec<ScriptExtensionSummary>,
    ) -> Result<BootstrapPayload, AppError> {
        let settings = self.load_settings()?;

        Ok(BootstrapPayload {
            health: AppHealth {
                profile: self.active_profile()?,
                command_count: commands.len(),
                tray_ready,
                storage_ready: true,
                extension_directory,
            },
            settings: settings.clone(),
            profiles: self.list_profiles()?,
            recent_history: self.list_recent_history(settings.history_limit.min(10))?,
            command_usage: self.command_usage()?,
            pinned_modules: self.list_pinned_modules(&commands)?,
            extensions,
            commands,
        })
    }

    pub fn update_settings(&self, payload: UpdateSettingsPayload) -> Result<AppSettings, AppError> {
        let connection = self.connect()?;
        connection.execute(
            "UPDATE app_settings
             SET theme_mode = ?1, launch_hotkey = ?2, close_to_tray = ?3, history_limit = ?4
             WHERE id = 1",
            params![
                payload.theme_mode,
                payload.launch_hotkey,
                payload.close_to_tray as i64,
                payload.history_limit
            ],
        )?;

        self.load_settings()
    }

    pub fn record_command_execution(
        &self,
        payload: RecordCommandExecutionPayload,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        let connection = self.connect()?;
        connection.execute(
            "INSERT INTO command_history (command_id, query_text, input_text, executed_at)
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![payload.command_id, payload.query_text, payload.input_text],
        )?;

        let history_limit = self.load_settings()?.history_limit;
        connection.execute(
            "DELETE FROM command_history
             WHERE id NOT IN (
               SELECT id FROM command_history ORDER BY id DESC LIMIT ?1
             )",
            params![history_limit],
        )?;

        self.list_recent_history(history_limit.min(10))
    }

    pub fn search_history(
        &self,
        payload: SearchHistoryPayload,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        let connection = self.connect()?;
        let limit = payload.limit.clamp(1, 100);
        let query = format!("%{}%", payload.query_text.trim().to_lowercase());
        let mut statement = connection.prepare(
            "SELECT id, command_id, query_text, input_text, executed_at
             FROM command_history
             WHERE lower(command_id) LIKE ?1
                OR lower(query_text) LIKE ?1
                OR lower(input_text) LIKE ?1
             ORDER BY id DESC
             LIMIT ?2",
        )?;

        let rows = statement.query_map(params![query, limit], |row| {
            Ok(CommandHistoryEntry {
                id: row.get(0)?,
                command_id: row.get(1)?,
                query_text: row.get(2)?,
                input_text: row.get(3)?,
                executed_at: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn set_active_profile(&self, profile_id: &str) -> Result<WorkspaceProfile, AppError> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;

        let exists: Option<String> = transaction
            .query_row(
                "SELECT id FROM workspace_profiles WHERE id = ?1",
                params![profile_id],
                |row| row.get(0),
            )
            .optional()?;

        if exists.is_none() {
            return Err(AppError::Internal(format!("profile does not exist: {profile_id}")));
        }

        transaction.execute("UPDATE workspace_profiles SET is_active = 0", [])?;
        transaction.execute(
            "UPDATE workspace_profiles SET is_active = 1 WHERE id = ?1",
            params![profile_id],
        )?;
        let hotkey: String = transaction.query_row(
            "SELECT launch_hotkey FROM workspace_profiles WHERE id = ?1",
            params![profile_id],
            |row| row.get(0),
        )?;
        transaction.execute(
            "UPDATE app_settings SET launch_hotkey = ?1 WHERE id = 1",
            params![hotkey],
        )?;
        transaction.commit()?;

        self.active_profile()
    }

    pub fn save_profile(&self, payload: SaveWorkspaceProfilePayload) -> Result<WorkspaceProfile, AppError> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let profile_id = payload
            .id
            .clone()
            .unwrap_or_else(|| format!("profile-{}", sanitize_profile_name(&payload.name)));
        let environment_tags = serde_json::to_string(&payload.environment_tags)?;
        let enabled_categories = serde_json::to_string(&app::default_profile_categories())?;
        let enabled_command_ids = serde_json::to_string(&payload.enabled_command_ids)?;

        let exists: Option<String> = transaction
            .query_row(
                "SELECT id FROM workspace_profiles WHERE id = ?1",
                params![profile_id],
                |row| row.get(0),
            )
            .optional()?;

        if exists.is_some() {
            transaction.execute(
                "UPDATE workspace_profiles
                 SET name = ?2,
                     environment_tags = ?3,
                     enabled_categories = ?4,
                     enabled_command_ids = ?5,
                     default_command_id = ?6,
                     launch_hotkey = ?7
                 WHERE id = ?1",
                params![
                    profile_id,
                    payload.name,
                    environment_tags,
                    enabled_categories,
                    enabled_command_ids,
                    payload.default_command_id,
                    payload.launch_hotkey
                ],
            )?;
        } else {
            transaction.execute(
                "INSERT INTO workspace_profiles (
                    id, name, environment_tags, enabled_categories, enabled_command_ids,
                    default_command_id, launch_hotkey, is_default, is_active
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 0)",
                params![
                    profile_id,
                    payload.name,
                    environment_tags,
                    enabled_categories,
                    enabled_command_ids,
                    payload.default_command_id,
                    payload.launch_hotkey
                ],
            )?;
        }

        transaction.commit()?;
        self.profile_by_id(&profile_id)
    }

    pub fn upsert_pinned_module(&self, command_id: &str, title: &str) -> Result<(), AppError> {
        let connection = self.connect()?;
        connection.execute(
            "INSERT OR REPLACE INTO pinned_modules (command_id, title, window_label)
             VALUES (?1, ?2, ?3)",
            params![command_id, title, app::module_window_label(command_id)],
        )?;
        Ok(())
    }

    pub fn remove_pinned_module(&self, command_id: &str) -> Result<(), AppError> {
        let connection = self.connect()?;
        connection.execute(
            "DELETE FROM pinned_modules WHERE command_id = ?1",
            params![command_id],
        )?;
        Ok(())
    }

    pub fn configuration_snapshot(&self) -> Result<ConfigurationSnapshot, AppError> {
        Ok(ConfigurationSnapshot {
            settings: self.load_settings()?,
            profiles: self.list_profiles()?,
        })
    }

    pub fn import_configuration(&self, snapshot: ConfigurationSnapshot) -> Result<WorkspaceProfile, AppError> {
        if snapshot.profiles.is_empty() {
            return Err(AppError::Internal(
                "configuration import requires at least one workspace profile".into(),
            ));
        }

        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        transaction.execute("DELETE FROM workspace_profiles", [])?;
        transaction.execute("DELETE FROM app_settings", [])?;

        let active_profile = snapshot
            .profiles
            .iter()
            .find(|profile| profile.is_default)
            .cloned()
            .unwrap_or_else(|| snapshot.profiles[0].clone());

        for profile in &snapshot.profiles {
            transaction.execute(
                "INSERT INTO workspace_profiles (
                    id, name, environment_tags, enabled_categories, enabled_command_ids,
                    default_command_id, launch_hotkey, is_default, is_active
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    profile.id,
                    profile.name,
                    serde_json::to_string(&profile.environment_tags)?,
                    serde_json::to_string(&profile.enabled_categories)?,
                    serde_json::to_string(&profile.enabled_command_ids)?,
                    profile.default_command_id,
                    profile.launch_hotkey,
                    profile.is_default as i64,
                    (profile.id == active_profile.id) as i64
                ],
            )?;
        }

        transaction.execute(
            "INSERT INTO app_settings (id, theme_mode, launch_hotkey, close_to_tray, history_limit)
             VALUES (1, ?1, ?2, ?3, ?4)",
            params![
                snapshot.settings.theme_mode,
                active_profile.launch_hotkey,
                snapshot.settings.close_to_tray as i64,
                snapshot.settings.history_limit
            ],
        )?;

        transaction.commit()?;
        self.active_profile()
    }

    pub fn write_text_file(&self, path: &str, contents: &str) -> Result<(), AppError> {
        fs::write(path, contents)
            .map_err(|error| AppError::Internal(format!("failed to write file {path}: {error}")))
    }

    pub fn read_text_file(&self, path: &str) -> Result<String, AppError> {
        fs::read_to_string(path)
            .map_err(|error| AppError::Internal(format!("failed to read file {path}: {error}")))
    }

    pub fn remove_pinned_module_by_window_label(&self, window_label: &str) -> Result<(), AppError> {
        let connection = self.connect()?;
        connection.execute(
            "DELETE FROM pinned_modules WHERE window_label = ?1",
            params![window_label],
        )?;
        Ok(())
    }

    fn initialize(&self) -> Result<(), AppError> {
        let connection = self.connect()?;
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS workspace_profiles (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              environment_tags TEXT NOT NULL DEFAULT '[]',
              enabled_categories TEXT NOT NULL,
              enabled_command_ids TEXT NOT NULL DEFAULT '[]',
              default_command_id TEXT NOT NULL DEFAULT 'data.format-json',
              launch_hotkey TEXT NOT NULL DEFAULT 'Alt+Space',
              is_default INTEGER NOT NULL DEFAULT 0,
              is_active INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS app_settings (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              theme_mode TEXT NOT NULL,
              launch_hotkey TEXT NOT NULL,
              close_to_tray INTEGER NOT NULL DEFAULT 0,
              history_limit INTEGER NOT NULL DEFAULT 50
            );

            CREATE TABLE IF NOT EXISTS command_history (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              command_id TEXT NOT NULL,
              query_text TEXT NOT NULL DEFAULT '',
              input_text TEXT NOT NULL DEFAULT '',
              executed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pinned_modules (
              command_id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              window_label TEXT NOT NULL
            );
            ",
        )?;

        self.migrate_workspace_profiles(&connection)?;
        ensure_column(&connection, "command_history", "input_text", "TEXT NOT NULL DEFAULT ''")?;
        self.seed_defaults(&connection)?;
        Ok(())
    }

    fn seed_defaults(&self, connection: &Connection) -> Result<(), AppError> {
        let profile_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM workspace_profiles", [], |row| row.get(0))?;

        if profile_count == 0 {
            let environment_tags = serde_json::to_string(&vec!["general"])?;
            let enabled_categories = serde_json::to_string(&app::default_profile_categories())?;
            let enabled_command_ids = serde_json::to_string(&app::default_command_ids())?;
            connection.execute(
                "INSERT INTO workspace_profiles (
                    id, name, environment_tags, enabled_categories, enabled_command_ids,
                    default_command_id, launch_hotkey, is_default, is_active
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 1)",
                params![
                    app::DEFAULT_PROFILE_ID,
                    app::default_profile_name(),
                    environment_tags,
                    enabled_categories,
                    enabled_command_ids,
                    "data.format-json",
                    app::platform_default_hotkey()
                ],
            )?;
        }

        let settings_exists: Option<i64> = connection
            .query_row("SELECT id FROM app_settings WHERE id = 1", [], |row| row.get(0))
            .optional()?;

        if settings_exists.is_none() {
            connection.execute(
                "INSERT INTO app_settings (id, theme_mode, launch_hotkey, close_to_tray, history_limit)
                 VALUES (1, ?1, ?2, 0, 50)",
                params!["system", app::platform_default_hotkey()],
            )?;
        }

        Ok(())
    }

    fn active_profile(&self) -> Result<WorkspaceProfile, AppError> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT id, name, environment_tags, enabled_categories, enabled_command_ids, default_command_id, launch_hotkey, is_default
                 FROM workspace_profiles
                 WHERE is_active = 1
                 LIMIT 1",
                [],
                map_profile_row,
            )
            .map_err(Into::into)
    }

    fn list_profiles(&self) -> Result<Vec<WorkspaceProfile>, AppError> {
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT id, name, environment_tags, enabled_categories, enabled_command_ids, default_command_id, launch_hotkey, is_default
             FROM workspace_profiles
             ORDER BY is_default DESC, name ASC",
        )?;

        let rows = statement.query_map([], map_profile_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn load_settings(&self) -> Result<AppSettings, AppError> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT theme_mode, launch_hotkey, close_to_tray, history_limit
                 FROM app_settings
                 WHERE id = 1",
                [],
                |row| {
                    Ok(AppSettings {
                        theme_mode: row.get(0)?,
                        launch_hotkey: row.get(1)?,
                        close_to_tray: row.get::<_, i64>(2)? != 0,
                        history_limit: row.get(3)?,
                    })
                },
            )
            .map_err(Into::into)
    }

    fn list_recent_history(&self, limit: u32) -> Result<Vec<CommandHistoryEntry>, AppError> {
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT id, command_id, query_text, input_text, executed_at
             FROM command_history
             ORDER BY id DESC
             LIMIT ?1",
        )?;

        let rows = statement.query_map(params![limit], |row| {
            Ok(CommandHistoryEntry {
                id: row.get(0)?,
                command_id: row.get(1)?,
                query_text: row.get(2)?,
                input_text: row.get(3)?,
                executed_at: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn connect(&self) -> Result<Connection, AppError> {
        Connection::open(&self.db_path).map_err(Into::into)
    }

    fn profile_by_id(&self, profile_id: &str) -> Result<WorkspaceProfile, AppError> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT id, name, environment_tags, enabled_categories, enabled_command_ids, default_command_id, launch_hotkey, is_default
                 FROM workspace_profiles
                 WHERE id = ?1",
                params![profile_id],
                map_profile_row,
            )
            .map_err(Into::into)
    }

    fn command_usage(&self) -> Result<Vec<CommandUsageEntry>, AppError> {
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT command_id, COUNT(*) as execution_count
             FROM command_history
             GROUP BY command_id",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(CommandUsageEntry {
                command_id: row.get(0)?,
                execution_count: row.get(1)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn list_pinned_modules(&self, commands: &[CommandAction]) -> Result<Vec<PinnedModule>, AppError> {
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT command_id, window_label, title
             FROM pinned_modules
             ORDER BY title ASC",
        )?;

        let rows = statement.query_map([], |row| {
            let command_id: String = row.get(0)?;
            let title = commands
                .iter()
                .find(|command| command.id == command_id)
                .map(|command| command.title.clone())
                .unwrap_or_else(|| row.get::<_, String>(2).unwrap_or_else(|_| command_id.clone()));

            Ok(PinnedModule {
                command_id,
                window_label: row.get(1)?,
                title,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn migrate_workspace_profiles(&self, connection: &Connection) -> Result<(), AppError> {
        ensure_column(connection, "workspace_profiles", "environment_tags", "TEXT NOT NULL DEFAULT '[]'")?;
        ensure_column(connection, "workspace_profiles", "enabled_command_ids", "TEXT NOT NULL DEFAULT '[]'")?;
        ensure_column(
            connection,
            "workspace_profiles",
            "default_command_id",
            "TEXT NOT NULL DEFAULT 'data.format-json'",
        )?;
        ensure_column(
            connection,
            "workspace_profiles",
            "launch_hotkey",
            &format!("TEXT NOT NULL DEFAULT '{}'", app::platform_default_hotkey()),
        )?;
        Ok(())
    }
}

fn map_profile_row(row: &rusqlite::Row<'_>) -> Result<WorkspaceProfile, rusqlite::Error> {
    let environment_tags: String = row.get(2)?;
    let environment_tags = serde_json::from_str(&environment_tags).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let enabled_categories: String = row.get(3)?;
    let enabled_categories = serde_json::from_str(&enabled_categories).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let enabled_command_ids: String = row.get(4)?;
    let enabled_command_ids = serde_json::from_str(&enabled_command_ids).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(WorkspaceProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        environment_tags,
        enabled_categories,
        enabled_command_ids,
        default_command_id: row.get(5)?,
        launch_hotkey: row.get(6)?,
        is_default: row.get::<_, i64>(7)? != 0,
    })
}

fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), AppError> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut statement = connection.prepare(&pragma)?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let existing: String = row.get(1)?;
        if existing == column {
            return Ok(());
        }
    }

    let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
    connection.execute(&alter, [])?;
    Ok(())
}

fn sanitize_profile_name(name: &str) -> String {
    let sanitized = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|char| if char.is_ascii_alphanumeric() { char } else { '-' })
        .collect::<String>();
    sanitized.trim_matches('-').to_string()
}
