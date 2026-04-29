use async_trait::async_trait;
use nexis_plugin::{Plugin, Command, Response, PluginError, Message, Member};

pub struct TranslatePlugin;

impl TranslatePlugin {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Plugin for TranslatePlugin {
    fn name(&self) -> &str { "translate" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_command(&self, cmd: &Command) -> Result<Option<Response>, PluginError> {
        if cmd.name != "translate" {
            return Ok(None);
        }
        if cmd.args.len() < 2 {
            return Ok(Some(Response {
                content: "Usage: /translate <text> <target_lang>\nExample: /translate Hello zh".to_string(),
                is_private: true,
            }));
        }
        let lang_idx = cmd.args.len() - 1;
        let text = cmd.args[..lang_idx].join(" ");
        let target_lang = &cmd.args[lang_idx];
        Ok(Some(Response {
            content: format!("🌐 [{}→{}] {} (configure TRANSLATE_ENGINE for live translation)", "auto", target_lang, text),
            is_private: false,
        }))
    }
}
