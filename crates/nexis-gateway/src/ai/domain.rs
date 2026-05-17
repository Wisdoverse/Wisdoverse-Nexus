//! AI chat domain policy.

const DEFAULT_TRIGGERS: [&str; 2] = ["ai", "assistant"];

/// A detected AI mention in a chat message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiMention {
    /// Mention trigger without the leading `@`.
    pub trigger: String,
}

/// AI response ready to be rendered as a chat message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiResponse {
    /// The response content.
    pub content: String,
    /// The agent name to display.
    pub agent_name: String,
    /// The model used.
    pub model: Option<String>,
}

impl AiResponse {
    /// Format the response as a chat message.
    pub fn to_chat_message(&self) -> String {
        format!("[{}]: {}", self.agent_name, self.content)
    }
}

pub fn default_triggers() -> Vec<String> {
    DEFAULT_TRIGGERS.iter().map(ToString::to_string).collect()
}

/// Detect if a message contains an AI mention.
///
/// Supports formats like `@ai`, `@AI`, and configured trigger names. Uses a word
/// boundary check to avoid false positives like `@airplane`.
pub fn detect_ai_mention(message: &str, triggers: &[String]) -> Option<AiMention> {
    for word in message.split_whitespace() {
        let lower_word = word.to_lowercase();
        for trigger in triggers {
            let trigger = trigger.to_lowercase();
            let pattern = format!("@{trigger}");
            if lower_word == pattern {
                return Some(AiMention { trigger });
            }
        }
    }

    None
}

/// Extract the prompt by removing configured AI mentions while preserving case.
pub fn extract_prompt(message: &str, triggers: &[String]) -> String {
    let mut cleaned = message.to_string();

    for trigger in triggers {
        let mut result = String::new();
        let mut last_end = 0;
        let lower_msg = message.to_lowercase();
        let pattern = format!("@{}", trigger.to_lowercase());

        let mut start = 0;
        while start < message.len() {
            if let Some(pos) = lower_msg[start..].find(&pattern) {
                let match_start = start + pos;
                let match_end = match_start + pattern.len();

                let is_word_boundary = match_end >= message.len()
                    || !message.as_bytes()[match_end].is_ascii_alphanumeric();

                if is_word_boundary {
                    result.push_str(&message[last_end..match_start]);
                    last_end = match_end;
                }
                start = match_end;
            } else {
                break;
            }
        }

        if last_end > 0 {
            result.push_str(&message[last_end..]);
            cleaned = result;
        }
    }

    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ai_mention() {
        let triggers = default_triggers();

        assert!(detect_ai_mention("@ai hello", &triggers).is_some());
        assert!(detect_ai_mention("@AI what's up", &triggers).is_some());
        assert!(detect_ai_mention("Hey @assistant help", &triggers).is_some());
        assert!(detect_ai_mention("@ASSISTANT please", &triggers).is_some());
        assert!(detect_ai_mention("hello @ai", &triggers).is_some());
    }

    #[test]
    fn ignores_non_mentions() {
        let triggers = default_triggers();

        assert!(detect_ai_mention("hello world", &triggers).is_none());
        assert!(detect_ai_mention("email@test.com", &triggers).is_none());
        assert!(detect_ai_mention("someone@example.com", &triggers).is_none());
        assert!(detect_ai_mention("@airplane is flying", &triggers).is_none());
        assert!(detect_ai_mention("@aid is needed", &triggers).is_none());
        assert!(detect_ai_mention("@ailing health", &triggers).is_none());
    }

    #[test]
    fn extracts_prompt() {
        let triggers = default_triggers();

        assert_eq!(extract_prompt("@ai hello", &triggers), "hello");
        assert_eq!(extract_prompt("hello @ai", &triggers), "hello");
        assert_eq!(extract_prompt("@AI what's up", &triggers), "what's up");
        assert_eq!(
            extract_prompt("Hey @assistant help me", &triggers),
            "Hey  help me"
        );
        assert_eq!(
            extract_prompt("Hello World @ai How Are You?", &triggers),
            "Hello World  How Are You?"
        );
        assert_eq!(
            extract_prompt("@airplane is cool @ai but this is removed", &triggers),
            "@airplane is cool  but this is removed"
        );
    }

    #[test]
    fn formats_ai_response_as_chat_message() {
        let response = AiResponse {
            content: "Hello! How can I help?".to_string(),
            agent_name: "ai".to_string(),
            model: Some("gpt-4o-mini".to_string()),
        };

        assert_eq!(response.to_chat_message(), "[ai]: Hello! How can I help?");
    }
}
