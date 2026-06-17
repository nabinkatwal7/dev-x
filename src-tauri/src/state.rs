use std::sync::Arc;

use crate::{
    error::AppError,
    services::{
        command_executor::CommandExecutor, command_registry::CommandRegistry,
        extension_loader::ExtensionLoader, storage::StorageService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub command_registry: Arc<CommandRegistry>,
    pub command_executor: Arc<CommandExecutor>,
    pub extension_loader: Arc<ExtensionLoader>,
    pub storage: Arc<StorageService>,
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Result<Self, AppError> {
        let storage = StorageService::new(&app_handle)?;
        let extension_loader = ExtensionLoader::new(&app_handle)?;
        extension_loader.start_watcher();

        Ok(Self {
            command_registry: Arc::new(CommandRegistry::new()),
            command_executor: Arc::new(CommandExecutor::new()),
            extension_loader: Arc::new(extension_loader),
            storage: Arc::new(storage),
        })
    }

    pub fn commands(&self) -> Vec<crate::models::CommandAction> {
        let mut commands = self.command_registry.commands().to_vec();
        commands.extend(self.extension_loader.command_actions());
        commands
    }
}
