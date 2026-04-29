//! Media routing interfaces and default implementation.

use uuid::Uuid;

use super::MediaTrack;

/// Interface for forwarding media between participants.
pub trait MediaRouter {
    /// Compute recipient participants for media from `sender_id` on `track`.
    fn route_media(sender_id: Uuid, track: MediaTrack, participants: &[Uuid]) -> Vec<Uuid>;
    /// React to a keyframe request from a participant for a specific track.
    fn handle_keyframe(participant_id: Uuid, track: MediaTrack);
}

/// Default in-memory media router behavior.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultMediaRouter;

impl MediaRouter for DefaultMediaRouter {
    fn route_media(sender_id: Uuid, _track: MediaTrack, participants: &[Uuid]) -> Vec<Uuid> {
        participants
            .iter()
            .copied()
            .filter(|participant_id| participant_id != &sender_id)
            .collect()
    }

    fn handle_keyframe(_participant_id: Uuid, _track: MediaTrack) {}
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::sfu::MediaTrack;

    use super::{DefaultMediaRouter, MediaRouter};

    #[test]
    fn route_media_excludes_sender_and_returns_other_participants() {
        let sender = Uuid::new_v4();
        let recipient_a = Uuid::new_v4();
        let recipient_b = Uuid::new_v4();

        let recipients = DefaultMediaRouter::route_media(
            sender,
            MediaTrack::Video,
            &[sender, recipient_a, recipient_b],
        );

        assert_eq!(recipients.len(), 2);
        assert!(recipients.contains(&recipient_a));
        assert!(recipients.contains(&recipient_b));
        assert!(!recipients.contains(&sender));
    }

    #[test]
    fn handle_keyframe_is_noop_for_now() {
        let participant_id = Uuid::new_v4();
        DefaultMediaRouter::handle_keyframe(participant_id, MediaTrack::ScreenShare);
    }
}
