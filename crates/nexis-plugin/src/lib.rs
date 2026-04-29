pub mod error;
pub mod extension_points;
pub mod manager;
pub mod plugin;

pub use error::PluginError;
pub use extension_points::{CommandHandler, MessageFilter, NotificationChannel, StorageAdapter};
pub use manager::PluginManager;
pub use plugin::{Command, Member, MemberType, Message, MessageType, Plugin, Response};
