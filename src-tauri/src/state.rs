use std::sync::Arc;

use tauri::AppHandle;

use crate::services::command_registry::CommandRegistry;

#[derive(Clone)]
pub struct AppState {
    pub app_handle: AppHandle,
    pub command_registry: Arc<CommandRegistry>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            command_registry: Arc::new(CommandRegistry::new()),
        }
    }
}
