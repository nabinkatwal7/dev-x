use std::{fs, path::PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, Manager};

use crate::{
    app,
    error::AppError,
    models::{
        AppHealth, AppSettings, BootstrapPayload, CommandAction, CommandHistoryEntry, CommandUsageEntry,
        RecordCommandExecutionPayload, UpdateSettingsPayload, WorkspaceProfile,
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
    ) -> Result<BootstrapPayload, AppError> {
        let settings = self.load_settings()?;

        Ok(BootstrapPayload {
            health: AppHealth {
                profile: self.active_profile()?,
                command_count: commands.len(),
                tray_ready,
                storage_ready: true,
            },
            settings: settings.clone(),
            profiles: self.list_profiles()?,
            recent_history: self.list_recent_history(settings.history_limit.min(10))?,
            command_usage: self.command_usage()?,
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
            "INSERT INTO command_history (command_id, query_text, executed_at)
             VALUES (?1, ?2, datetime('now'))",
            params![payload.command_id, payload.query_text],
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
        transaction.commit()?;

        self.active_profile()
    }

    fn initialize(&self) -> Result<(), AppError> {
        let connection = self.connect()?;
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS workspace_profiles (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              enabled_categories TEXT NOT NULL,
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
              executed_at TEXT NOT NULL
            );
            ",
        )?;

        self.seed_defaults(&connection)?;
        Ok(())
    }

    fn seed_defaults(&self, connection: &Connection) -> Result<(), AppError> {
        let profile_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM workspace_profiles", [], |row| row.get(0))?;

        if profile_count == 0 {
            let enabled_categories = serde_json::to_string(&app::default_profile_categories())?;
            connection.execute(
                "INSERT INTO workspace_profiles (id, name, enabled_categories, is_default, is_active)
                 VALUES (?1, ?2, ?3, 1, 1)",
                params![
                    app::DEFAULT_PROFILE_ID,
                    app::default_profile_name(),
                    enabled_categories
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
                "SELECT id, name, enabled_categories, is_default
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
            "SELECT id, name, enabled_categories, is_default
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
            "SELECT id, command_id, query_text, executed_at
             FROM command_history
             ORDER BY id DESC
             LIMIT ?1",
        )?;

        let rows = statement.query_map(params![limit], |row| {
            Ok(CommandHistoryEntry {
                id: row.get(0)?,
                command_id: row.get(1)?,
                query_text: row.get(2)?,
                executed_at: row.get(3)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn connect(&self) -> Result<Connection, AppError> {
        Connection::open(&self.db_path).map_err(Into::into)
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
}

fn map_profile_row(row: &rusqlite::Row<'_>) -> Result<WorkspaceProfile, rusqlite::Error> {
    let enabled_categories: String = row.get(2)?;
    let enabled_categories = serde_json::from_str(&enabled_categories).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(WorkspaceProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        enabled_categories,
        is_default: row.get::<_, i64>(3)? != 0,
    })
}
