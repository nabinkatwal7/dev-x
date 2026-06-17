use std::sync::Arc;

use crate::{
    error::AppError,
    services::{command_registry::CommandRegistry, storage::StorageService},
};

#[derive(Clone)]
pub struct AppState {
    pub command_registry: Arc<CommandRegistry>,
    pub storage: Arc<StorageService>,
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Result<Self, AppError> {
        let storage = StorageService::new(&app_handle)?;

        Ok(Self {
            command_registry: Arc::new(CommandRegistry::new()),
            storage: Arc::new(storage),
        })
    }
}
