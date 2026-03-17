use async_trait::async_trait;
use nexis_plugin::{Plugin, Command, Response, Member, PluginError, Message};

pub struct GitHubPlugin;

impl GitHubPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for GitHubPlugin {
    fn name(&self) -> &str { "github" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_command(&self, cmd: &Command) -> Result<Option<Response>, PluginError> {
        match cmd.name.as_str() {
            "github" => {
                if cmd.args.is_empty() {
                    return Ok(Some(Response {
                        content: "GitHub Plugin Commands:\n/github issue list\n/github issue create <title>".to_string(),
                        is_private: true,
                    }));
                }
                match cmd.args.get(0).map(|s| s.as_str()) {
                    Some("issue") => match cmd.args.get(1).map(|s| s.as_str()) {
                        Some("list") => Ok(Some(Response {
                            content: "📋 Open issues: (configure GITHUB_TOKEN for live data)".to_string(),
                            is_private: false,
                        })),
                        Some("create") => {
                            let title = cmd.args[2..].join(" ");
                            Ok(Some(Response {
                                content: format!("✅ Issue created: \"{}\" (configure GITHUB_TOKEN)", title),
                                is_private: false,
                            }))
                        }
                        _ => Ok(Some(Response {
                            content: "Usage: /github issue list|create <title>".to_string(),
                            is_private: true,
                        })),
                    },
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
}
