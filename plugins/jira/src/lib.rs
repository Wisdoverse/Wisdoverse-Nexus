use async_trait::async_trait;
use nexis_plugin::{Plugin, Command, Response, PluginError, Message, Member};

pub struct JiraPlugin;

impl JiraPlugin {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Plugin for JiraPlugin {
    fn name(&self) -> &str { "jira" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_command(&self, cmd: &Command) -> Result<Option<Response>, PluginError> {
        match cmd.name.as_str() {
            "jira" => {
                if cmd.args.is_empty() {
                    return Ok(Some(Response {
                        content: "Jira Commands:\n/jira create <summary>\n/jira list\n/jira sprint".to_string(),
                        is_private: true,
                    }));
                }
                match cmd.args.get(0).map(|s| s.as_str()) {
                    Some("create") => {
                        let summary = cmd.args[1..].join(" ");
                        Ok(Some(Response {
                            content: format!("✅ Jira issue created: \"{}\" (configure JIRA_API_TOKEN)", summary),
                            is_private: false,
                        }))
                    }
                    Some("list") => Ok(Some(Response {
                        content: "📋 Open Jira issues: (configure JIRA_API_TOKEN for live data)".to_string(),
                        is_private: false,
                    })),
                    Some("sprint") => Ok(Some(Response {
                        content: "🏃 Sprint status: (configure JIRA_API_TOKEN for live data)".to_string(),
                        is_private: false,
                    })),
                    _ => Ok(Some(Response {
                        content: "Usage: /jira create|list|sprint".to_string(),
                        is_private: true,
                    })),
                }
            }
            _ => Ok(None),
        }
    }
}
