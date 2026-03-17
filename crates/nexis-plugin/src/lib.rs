pub mod error;
pub mod plugin;
pub mod extension_points;
pub mod manager;

pub use error::PluginError;
pub use plugin::{Plugin, Message, Member, Command, Response, MessageType, MemberType};
pub use manager::PluginManager;
pub use extension_points::{MessageFilter, CommandHandler, NotificationChannel, StorageAdapter};
