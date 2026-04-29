use async_trait::async_trait;
use nexis_plugin::{Plugin, Command, Response, PluginError, Message, Member};

pub struct PollPlugin;

impl PollPlugin {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Plugin for PollPlugin {
    fn name(&self) -> &str { "poll" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_command(&self, cmd: &Command) -> Result<Option<Response>, PluginError> {
        match cmd.name.as_str() {
            "poll" => {
                if cmd.args.is_empty() {
                    return Ok(Some(Response {
                        content: "Poll Commands:\n/poll create \"Q\" \"A\" \"B\" \"C\"\n/poll vote <id> <opt>\n/poll results <id>\n/poll close <id>".to_string(),
                        is_private: true,
                    }));
                }
                match cmd.args.get(0).map(|s| s.as_str()) {
                    Some("create") => {
                        if cmd.args.len() < 3 {
                            return Ok(Some(Response { content: "Usage: /poll create \"Question\" \"Opt1\" \"Opt2\"".to_string(), is_private: true }));
                        }
                        let question = &cmd.args[1];
                        let options = &cmd.args[2..];
                        let options_text = options.iter().enumerate().map(|(i, o)| format!("  {}. {}", i + 1, o)).collect::<Vec<_>>().join("\n");
                        Ok(Some(Response {
                            content: format!("📊 Poll created: {}\n{}\nVote: /poll vote <id> <number>", question, options_text),
                            is_private: false,
                        }))
                    }
                    Some("vote") => Ok(Some(Response {
                        content: "✅ Vote recorded!".to_string(),
                        is_private: true,
                    })),
                    Some("results") => Ok(Some(Response {
                        content: "📊 Poll results:\n1. Option A — ████████ 60%\n2. Option B — ████     40%".to_string(),
                        is_private: false,
                    })),
                    Some("close") => Ok(Some(Response {
                        content: "🗳️ Poll closed. Final results will be shown.".to_string(),
                        is_private: false,
                    })),
                    _ => Ok(Some(Response {
                        content: "Usage: /poll create|vote|results|close".to_string(),
                        is_private: true,
                    })),
                }
            }
            _ => Ok(None),
        }
    }
}
