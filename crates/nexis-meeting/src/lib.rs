//! Wisdoverse Nexus Meeting - meeting rooms, signaling, recording, and summaries.

pub mod error;
pub mod participant;
pub mod recording;
pub mod room;
pub mod sfu;
pub mod signaling;
pub mod summary;

pub use error::{MeetingError, MeetingResult};
pub use participant::{MediaState, Participant, ParticipantRole};
pub use recording::{Recording, RecordingState, TrackType};
pub use room::{MeetingRoom, RoomConfig, RoomState};
pub use sfu::{DefaultMediaRouter, MediaRouter, MediaTrack, SfuConfig, SfuRoom};
pub use signaling::{SignalMessage, SignalType};
pub use summary::{ActionItem, MeetingSummary};

/// Prelude for common imports.
pub mod prelude {
    pub use crate::error::{MeetingError, MeetingResult};
    pub use crate::participant::{MediaState, Participant, ParticipantRole};
    pub use crate::recording::{Recording, RecordingState, TrackType};
    pub use crate::room::{MeetingRoom, RoomConfig, RoomState};
    pub use crate::sfu::{DefaultMediaRouter, MediaRouter, MediaTrack, SfuConfig, SfuRoom};
    pub use crate::signaling::{SignalMessage, SignalType};
    pub use crate::summary::{ActionItem, MeetingSummary};
}
