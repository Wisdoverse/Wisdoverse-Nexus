use crate::error::PluginError;
use crate::plugin::Message;
use async_trait::async_trait;

/// Message filter — content moderation, sensitive words
#[async_trait]
pub trait MessageFilter: Send + Sync {
    /// Return true if the message should be allowed
    async fn filter(&self, message: &mut Message) -> Result<bool, PluginError>;
}

/// Command handler — slash commands
#[async_trait]
pub trait CommandHandler: Send + Sync {
    fn command_name(&self) -> &str;
    fn description(&self) -> &str;

    async fn handle(
        &self,
        args: &[String],
        sender_id: &str,
        room_id: &str,
    ) -> Result<Option<String>, PluginError>;
}

/// Notification channel — Webhook, Email, Slack
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    fn channel_name(&self) -> &str;

    async fn send(&self, title: &str, body: &str, target: &str) -> Result<(), PluginError>;
}

/// Storage adapter — S3, MongoDB, Redis
#[async_trait]
pub trait StorageAdapter: Send + Sync {
    fn adapter_name(&self) -> &str;

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, PluginError>;
    async fn set(&self, key: &str, value: &[u8]) -> Result<(), PluginError>;
    async fn delete(&self, key: &str) -> Result<(), PluginError>;
}
