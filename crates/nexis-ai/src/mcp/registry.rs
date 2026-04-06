//! MCP provider factory and registry

use std::str::FromStr;

use crate::{AIProvider, ProviderError};

use super::{AnthropicProvider, GeminiProvider, OpenAIProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    OpenAI,
    Anthropic,
    Gemini,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderKind::OpenAI => "openai",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Gemini => "gemini",
        }
    }

    pub fn required_api_key_env(self) -> &'static str {
        match self {
            ProviderKind::OpenAI => "OPENAI_API_KEY",
            ProviderKind::Anthropic => "ANTHROPIC_API_KEY",
            ProviderKind::Gemini => "GEMINI_API_KEY",
        }
    }
}

impl FromStr for ProviderKind {
    type Err = ProviderError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" => Ok(ProviderKind::OpenAI),
            "anthropic" | "claude" => Ok(ProviderKind::Anthropic),
            "gemini" => Ok(ProviderKind::Gemini),
            _ => Err(ProviderError::Message(format!(
                "unsupported provider '{value}', expected one of: openai, anthropic, gemini"
            ))),
        }
    }
}

/// Create a provider with default settings
pub fn create_provider(kind: ProviderKind, api_key: impl Into<String>) -> Box<dyn AIProvider> {
    let key = api_key.into();

    match kind {
        ProviderKind::OpenAI => Box::new(OpenAIProvider::new(key, "https://api.openai.com/v1", "gpt-4")),
        ProviderKind::Anthropic => Box::new(AnthropicProvider::new(key, "https://api.anthropic.com", "claude-3-sonnet-20240229")),
        ProviderKind::Gemini => Box::new(GeminiProvider::new(key)),
    }
}

pub fn create_provider_from_env() -> Result<Box<dyn AIProvider>, ProviderError> {
    let provider_name = std::env::var("NEXIS_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    let kind = ProviderKind::from_str(&provider_name)?;

    let api_key_env = kind.required_api_key_env();
    let api_key = std::env::var(api_key_env).map_err(|_| {
        ProviderError::Message(format!(
            "missing required environment variable '{api_key_env}' for provider '{}'",
            kind.as_str()
        ))
    })?;

    if api_key.trim().is_empty() {
        return Err(ProviderError::Message(format!(
            "environment variable '{api_key_env}' is empty for provider '{}'",
            kind.as_str()
        )));
    }

    Ok(create_provider(kind, api_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_kind_parse() {
        assert_eq!(ProviderKind::from_str("openai").unwrap(), ProviderKind::OpenAI);
        assert_eq!(ProviderKind::from_str("anthropic").unwrap(), ProviderKind::Anthropic);
        assert_eq!(ProviderKind::from_str("claude").unwrap(), ProviderKind::Anthropic);
        assert_eq!(ProviderKind::from_str("gemini").unwrap(), ProviderKind::Gemini);
    }

    #[test]
    fn provider_kind_env_vars() {
        assert_eq!(ProviderKind::OpenAI.required_api_key_env(), "OPENAI_API_KEY");
        assert_eq!(ProviderKind::Anthropic.required_api_key_env(), "ANTHROPIC_API_KEY");
        assert_eq!(ProviderKind::Gemini.required_api_key_env(), "GEMINI_API_KEY");
    }
}
