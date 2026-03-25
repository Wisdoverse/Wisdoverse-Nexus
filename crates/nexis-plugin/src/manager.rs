use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::PluginError;
use crate::plugin::{Command, Member, Plugin, Response};

/// Manages plugin lifecycle, registration, and dispatch
pub struct PluginManager {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new plugin. Calls on_init; rolls back on failure.
    pub async fn register<P: Plugin + 'static>(&self, mut plugin: P) -> Result<(), PluginError> {
        let name = plugin.name().to_string();

        if self.plugins.read().await.contains_key(&name) {
            return Err(PluginError::InitFailed(format!(
                "Plugin '{}' already registered",
                name
            )));
        }

        plugin.on_init().await.map_err(|e| {
            PluginError::InitFailed(format!("Plugin '{}' init failed: {}", name, e))
        })?;

        info!(plugin = %name, version = plugin.version(), "Plugin registered");
        self.plugins.write().await.insert(name, Arc::new(plugin));
        Ok(())
    }

    /// Unregister a plugin by name
    pub async fn unregister(&self, name: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().await;
        if let Some(_plugin) = plugins.remove(name) {
            // Note: on_teardown would need mut access; skip for Arc safety
            info!(plugin = %name, "Plugin unregistered");
            Ok(())
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    /// List registered plugin names
    pub async fn list_plugins(&self) -> Vec<String> {
        self.plugins.read().await.keys().cloned().collect()
    }

    /// Dispatch a message through all plugins (error-isolated)
    pub async fn dispatch_message(&self, message: &mut crate::plugin::Message) {
        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            if let Err(e) = plugin.on_message(message) {
                warn!(plugin = %name, error = %e, "Plugin message hook failed");
            }
        }
    }

    /// Dispatch member join event
    pub async fn dispatch_member_join(&self, member: &Member, room_id: &str) {
        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            if let Err(e) = plugin.on_member_join(member, room_id) {
                warn!(plugin = %name, error = %e, "Plugin member_join hook failed");
            }
        }
    }

    /// Dispatch member leave event
    pub async fn dispatch_member_leave(&self, member: &Member, room_id: &str) {
        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            if let Err(e) = plugin.on_member_leave(member, room_id) {
                warn!(plugin = %name, error = %e, "Plugin member_leave hook failed");
            }
        }
    }

    /// Dispatch a command. Returns first non-None response.
    pub async fn dispatch_command(
        &self,
        command: &Command,
    ) -> Result<Option<Response>, PluginError> {
        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            match plugin.on_command(command) {
                Ok(Some(response)) => return Ok(Some(response)),
                Ok(None) => continue,
                Err(e) => {
                    warn!(plugin = %name, error = %e, "Plugin command hook failed");
                }
            }
        }
        Ok(None)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
