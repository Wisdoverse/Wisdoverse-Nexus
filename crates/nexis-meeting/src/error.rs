//! Error types for meeting operations.

use thiserror::Error;
use uuid::Uuid;

/// Result type for meeting operations.
pub type MeetingResult<T> = Result<T, MeetingError>;

/// Meeting crate error type.
#[derive(Debug, Error)]
pub enum MeetingError {
    /// Entity could not be found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Input validation failed.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Signaling operation failed.
    #[error("Signaling error: {0}")]
    Signaling(String),

    /// Recording operation failed.
    #[error("Recording error: {0}")]
    Recording(String),

    /// Room cannot accept more participants.
    #[error("Room capacity exceeded: max_participants={max_participants}")]
    RoomCapacityExceeded { max_participants: u16 },

    /// A participant identifier could not be resolved in room state.
    #[error("Participant not found in room: {participant_id}")]
    ParticipantNotFound { participant_id: Uuid },

    /// Requested codec is not supported for the media track.
    #[error("Unsupported codec `{codec}` for {media_track} track")]
    UnsupportedCodec { media_track: String, codec: String },

    /// Catch-all for irrecoverable internal failures.
    #[error("Internal meeting error: {0}")]
    Internal(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
