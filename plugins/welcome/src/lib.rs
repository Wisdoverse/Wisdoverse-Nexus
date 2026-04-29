use async_trait::async_trait;
use nexis_plugin::{Plugin, Command, Response, PluginError, Message, Member};

pub struct WelcomePlugin {
    template: String,
}

impl WelcomePlugin {
    pub fn new() -> Self {
        Self {
            template: "👋 Welcome to {room}, {user}! Feel free to ask anything.".to_string(),
        }
    }

    pub fn with_template(template: String) -> Self {
        Self { template }
    }
}

#[async_trait]
impl Plugin for WelcomePlugin {
    fn name(&self) -> &str { "welcome" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_member_join(&self, member: &Member, _room_id: &str) -> Result<(), PluginError> {
        let message = self.template
            .replace("{user}", &member.display_name)
            .replace("{room}", _room_id);
        tracing::info!(member = %member.id, "Welcome: {}", message);
        Ok(())
    }

    fn on_command(&self, cmd: &Command) -> Result<Option<Response>, PluginError> {
        if cmd.name != "welcome" {
            return Ok(None);
        }
        Ok(Some(Response {
            content: "Welcome plugin is active! New members will be greeted automatically.".to_string(),
            is_private: true,
        }))
    }
}
