//! Hello World example plugin for Wisdoverse Nexus.

use async_trait::async_trait;
use nexis_plugin::{Command, Member, Message, Plugin, PluginError, PluginManager, Response};

pub struct HelloWorldPlugin;

impl HelloWorldPlugin {
    pub fn new() -> Self { Self }
}

impl Default for HelloWorldPlugin {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Plugin for HelloWorldPlugin {
    fn name(&self) -> &str { "hello-world" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_message(&self, message: &mut Message) -> Result<(), PluginError> {
        if let nexis_protocol::MessageContent::Text { ref mut text } = message.content {
            if text.to_lowercase().contains("hello") {
                *text = format!("{} 👋", text);
            }
        }
        Ok(())
    }

    fn on_member_join(&self, member: &Member) -> Result<(), PluginError> {
        println!("[hello-world] Welcome {} to room {}!", member.display_name, member.room_id);
        Ok(())
    }

    fn on_member_leave(&self, member: &Member) -> Result<(), PluginError> {
        println!("[hello-world] Goodbye {} from room {}!", member.display_name, member.room_id);
        Ok(())
    }

    fn on_command(&self, cmd: &Command) -> Result<Option<Response>, PluginError> {
        match cmd.name.as_str() {
            "hello" => Ok(Some(Response {
                content: format!("Hello, {}! 🎉", cmd.sender_id),
                is_markdown: false,
            })),
            _ => Ok(None),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use nexis_protocol::{MemberId, MemberType, MessageContent};

    let manager = PluginManager::new();
    manager.register(HelloWorldPlugin::new()).await?;
    println!("Plugins: {:?}", manager.list_plugins().await);

    let mut message = Message {
        protocol_version: "1.0".into(),
        id: "msg-1".into(),
        room_id: "room-1".into(),
        sender: MemberId::new(MemberType::User, "user-1")?,
        content: MessageContent::Text { text: "hello world".into() },
        metadata: None,
        reply_to: None,
        created_at: chrono::Utc::now(),
        updated_at: None,
    };
    manager.dispatch_message(&mut message).await;
    println!("Filtered: {:?}", message.content);

    let cmd = Command {
        name: "hello".into(),
        args: vec![],
        sender_id: "user-1".into(),
        room_id: "room-1".into(),
    };
    if let Some(r) = manager.dispatch_command(&cmd).await {
        println!("Response: {}", r.content);
    }

    manager.unregister("hello-world").await?;
    Ok(())
}
