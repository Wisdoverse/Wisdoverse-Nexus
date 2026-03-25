//! AI-powered summarization implementation

use crate::error::{ContextError, ContextResult};
use crate::types::*;
use std::sync::Arc;
use tracing::{info, warn};

/// Configuration for AI Summarizer
#[derive(Debug, Clone)]
pub struct AISummarizerConfig {
    /// Maximum tokens for summary output
    pub max_output_tokens: usize,
    /// Temperature for generation (0.0-1.0)
    pub temperature: f32,
    /// Custom system prompt template
    pub system_prompt: Option<String>,
}

impl Default for AISummarizerConfig {
    fn default() -> Self {
        Self {
            max_output_tokens: 2048,
            temperature: 0.3,
            system_prompt: None,
        }
    }
}

/// AI-powered context summarizer
pub struct AISummarizer {
    ai_provider: Arc<dyn AiProvider>,
    config: AISummarizerConfig,
}

impl AISummarizer {
    pub fn new(ai_provider: Arc<dyn AiProvider>, config: AISummarizerConfig) -> Self {
        Self {
            ai_provider,
            config,
        }
    }

    pub fn with_defaults(ai_provider: Arc<dyn AiProvider>) -> Self {
        Self::new(ai_provider, AISummarizerConfig::default())
    }

    /// Summarize chat messages into a structured summary
    pub async fn summarize_messages(&self, messages: &[Message]) -> ContextResult<Summary> {
        if messages.is_empty() {
            return Ok(Summary::new("No messages to summarize.".to_string()));
        }

        let participants: Vec<String> = messages.iter().map(|m| m.sender_name.clone()).collect();
        let unique_participants: Vec<String> = participants
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let conversation_text = self.format_messages_for_summary(messages);
        let prompt = self.build_summary_prompt(&conversation_text, &unique_participants);

        match self.ai_provider.complete(&prompt).await {
            Ok(response) => self.parse_summary_response(&response, messages, unique_participants),
            Err(e) => {
                warn!("AI summarization failed: {}", e);
                Err(ContextError::SummarizationFailed(e.to_string()))
            }
        }
    }

    /// Generate meeting notes from meeting data
    pub async fn generate_meeting_notes(&self, meeting: &Meeting) -> ContextResult<MeetingNotes> {
        let duration_minutes = meeting
            .end_time
            .map(|end| ((end - meeting.start_time).num_minutes() as u32))
            .unwrap_or(0);

        let conversation_text = self.format_messages_for_summary(&meeting.messages);
        let prompt = self.build_meeting_notes_prompt(
            &meeting.title,
            &conversation_text,
            &meeting.participants,
            meeting.agenda.as_deref(),
        );

        match self.ai_provider.complete(&prompt).await {
            Ok(response) => self.parse_meeting_notes_response(
                &response,
                &meeting.title,
                &meeting.participants,
                duration_minutes,
            ),
            Err(e) => {
                warn!("Meeting notes generation failed: {}", e);
                Err(ContextError::SummarizationFailed(e.to_string()))
            }
        }
    }

    /// Ask questions about historical messages
    pub async fn ask_about_history(
        &self,
        question: &str,
        messages: &[Message],
    ) -> ContextResult<String> {
        if messages.is_empty() {
            return Ok("No conversation history available to answer this question.".to_string());
        }

        let conversation_text = self.format_messages_for_qa(messages);
        let prompt = format!(
            r#"You are a helpful assistant that answers questions about conversation history.

CONVERSATION HISTORY:
{}

QUESTION: {}

Based on the conversation history above, provide a clear and accurate answer to the question. If the answer cannot be found in the history, say so honestly."#,
            conversation_text, question
        );

        self.ai_provider
            .complete(&prompt)
            .await
            .map_err(|e| ContextError::SummarizationFailed(e.to_string()))
    }

    fn format_messages_for_summary(&self, messages: &[Message]) -> String {
        messages
            .iter()
            .map(|m| {
                format!(
                    "[{}] {}: {}",
                    m.timestamp.format("%H:%M"),
                    m.sender_name,
                    m.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_messages_for_qa(&self, messages: &[Message]) -> String {
        messages
            .iter()
            .map(|m| {
                format!(
                    "[{}] {}: {}",
                    m.timestamp.format("%Y-%m-%d %H:%M"),
                    m.sender_name,
                    m.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn build_summary_prompt(&self, conversation: &str, participants: &[String]) -> String {
        let system = self.config.system_prompt.as_deref().unwrap_or(
            "You are an expert conversation summarizer. Generate structured summaries in JSON format."
        );

        format!(
            r#"{}

PARTICIPANTS: {}

CONVERSATION:
{}

Generate a JSON summary with the following structure:
{{
  "summary": "A 2-3 sentence overview of the conversation",
  "key_points": ["List of 3-5 main discussion points"],
  "action_items": ["List of any action items or tasks mentioned"]
}}

Output ONLY valid JSON, no markdown formatting."#,
            system,
            participants.join(", "),
            conversation
        )
    }

    fn build_meeting_notes_prompt(
        &self,
        title: &str,
        conversation: &str,
        participants: &[String],
        agenda: Option<&[String]>,
    ) -> String {
        let agenda_text = agenda
            .map(|a| {
                format!(
                    "\nAGENDA:\n{}",
                    a.iter()
                        .map(|i| format!("- {}", i))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .unwrap_or_default();

        format!(
            r#"You are an expert meeting notes generator. Generate structured meeting notes in JSON format.

MEETING: {}
PARTICIPANTS: {}{}

CONVERSATION TRANSCRIPT:
{}

Generate meeting notes in JSON format:
{{
  "summary": "Brief overview of the meeting",
  "key_points": ["Main discussion points"],
  "decisions": ["Decisions made during the meeting"],
  "action_items": [
    {{"description": "Task description", "assignee": "Name", "priority": "high|medium|low"}}
  ],
  "next_meeting": "Suggested next meeting topic or null"
}}

Output ONLY valid JSON, no markdown formatting."#,
            title,
            participants.join(", "),
            agenda_text,
            conversation
        )
    }

    fn parse_summary_response(
        &self,
        response: &str,
        source_messages: &[Message],
        participants: Vec<String>,
    ) -> ContextResult<Summary> {
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        #[derive(serde::Deserialize)]
        struct RawSummary {
            summary: String,
            key_points: Vec<String>,
            action_items: Vec<String>,
        }

        let parsed: RawSummary = serde_json::from_str(cleaned).unwrap_or(RawSummary {
            summary: cleaned.to_string(),
            key_points: vec![],
            action_items: vec![],
        });

        Ok(Summary {
            summary: parsed.summary,
            key_points: parsed.key_points,
            action_items: parsed.action_items,
            participants,
            generated_at: chrono::Utc::now(),
            source_message_ids: source_messages.iter().map(|m| m.id.clone()).collect(),
            original_tokens: 0,
            summary_tokens: 0,
        })
    }

    fn parse_meeting_notes_response(
        &self,
        response: &str,
        title: &str,
        participants: &[String],
        duration_minutes: u32,
    ) -> ContextResult<MeetingNotes> {
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        #[derive(serde::Deserialize)]
        struct RawMeetingNotes {
            summary: String,
            key_points: Vec<String>,
            decisions: Vec<String>,
            action_items: Vec<RawActionItem>,
            next_meeting: Option<String>,
        }

        #[derive(serde::Deserialize)]
        struct RawActionItem {
            description: String,
            assignee: Option<String>,
            priority: Option<String>,
        }

        let parsed: RawMeetingNotes = serde_json::from_str(cleaned).unwrap_or(RawMeetingNotes {
            summary: cleaned.to_string(),
            key_points: vec![],
            decisions: vec![],
            action_items: vec![],
            next_meeting: None,
        });

        let action_items: Vec<ActionItem> = parsed
            .action_items
            .into_iter()
            .map(|a| ActionItem {
                description: a.description,
                assignee: a.assignee,
                due_date: None,
                priority: a.priority.and_then(|p| match p.to_lowercase().as_str() {
                    "high" => Some(Priority::High),
                    "medium" => Some(Priority::Medium),
                    "low" => Some(Priority::Low),
                    _ => None,
                }),
            })
            .collect();

        Ok(MeetingNotes {
            title: title.to_string(),
            summary: parsed.summary,
            key_points: parsed.key_points,
            decisions: parsed.decisions,
            action_items,
            participants: participants.to_vec(),
            duration_minutes,
            generated_at: chrono::Utc::now(),
            next_meeting: parsed.next_meeting,
        })
    }
}
