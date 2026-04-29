use crate::error::PluginError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A message flowing through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub room_id: String,
    pub sender_id: String,
    pub content: String,
    pub msg_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    System,
    File,
}

/// Room member info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub display_name: String,
    pub member_type: MemberType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberType {
    Human,
    Agent,
    Bot,
}

/// Command context passed to plugins
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub sender_id: String,
    pub room_id: String,
}

/// Plugin response to a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub content: String,
    pub is_private: bool,
}

/// Core plugin trait — all hooks have default no-op implementations
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    /// Called when the plugin is loaded
    async fn on_init(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called when the plugin is unloaded
    async fn on_teardown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Filter/transform a message before delivery
    fn on_message(&self, _message: &mut Message) -> Result<(), PluginError> {
        Ok(())
    }

    /// Handle member join event
    fn on_member_join(&self, _member: &Member, _room_id: &str) -> Result<(), PluginError> {
        Ok(())
    }

    /// Handle member leave event
    fn on_member_leave(&self, _member: &Member, _room_id: &str) -> Result<(), PluginError> {
        Ok(())
    }

    /// Handle a slash command
    fn on_command(&self, _command: &Command) -> Result<Option<Response>, PluginError> {
        Ok(None)
    }
}
