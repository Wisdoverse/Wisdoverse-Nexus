//! SFU types and in-memory room management.

mod router;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use router::{DefaultMediaRouter, MediaRouter};

/// Runtime SFU configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SfuConfig {
    /// Hard cap used by callers when admitting participants.
    pub max_participants: u16,
    /// Preferred video codec name (for example, `vp9`).
    pub video_codec: String,
    /// Preferred audio codec name (for example, `opus`).
    pub audio_codec: String,
}

/// Supported media tracks for publish/subscribe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaTrack {
    Audio,
    Video,
    ScreenShare,
}

/// In-memory SFU room state.
#[derive(Debug, Clone)]
pub struct SfuRoom {
    config: SfuConfig,
    participants: HashSet<Uuid>,
    published_tracks: HashMap<(Uuid, MediaTrack), Vec<u8>>,
    subscriptions: HashMap<Uuid, HashSet<MediaTrack>>,
}

impl SfuRoom {
    /// Create a new in-memory SFU room with the provided runtime configuration.
    pub fn new(config: SfuConfig) -> Self {
        Self {
            config,
            participants: HashSet::new(),
            published_tracks: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    /// Return immutable room configuration.
    pub fn config(&self) -> &SfuConfig {
        &self.config
    }

    /// Return the active participant identifiers currently in the room.
    pub fn participants(&self) -> &HashSet<Uuid> {
        &self.participants
    }

    /// Add a participant to the room and return the generated participant ID.
    ///
    /// Note: This method does not enforce capacity limits. Use [`Self::try_join_room`]
    /// for capacity-checked joins.
    pub fn join_room(&mut self) -> Uuid {
        let participant_id = Uuid::new_v4();
        self.participants.insert(participant_id);
        participant_id
    }

    /// Try to add a participant with capacity check.
    ///
    /// Returns `Ok(participant_id)` if the room has capacity, or
    /// `Err(MeetingError::RoomCapacityExceeded)` if at max_participants.
    pub fn try_join_room(&mut self) -> crate::error::MeetingResult<Uuid> {
        if self.participants.len() >= self.config.max_participants as usize {
            return Err(crate::error::MeetingError::RoomCapacityExceeded {
                max_participants: self.config.max_participants,
            });
        }
        Ok(self.join_room())
    }

    /// Remove a participant and any room state owned by that participant.
    ///
    /// Returns the participant ID that was requested to leave.
    pub fn leave_room(&mut self, participant_id: Uuid) -> Uuid {
        self.participants.remove(&participant_id);
        self.subscriptions.remove(&participant_id);
        self.published_tracks
            .retain(|(publisher_id, _), _| publisher_id != &participant_id);
        participant_id
    }

    /// Publish or replace the latest payload for a participant's media track.
    ///
    /// Publishing is ignored when the participant is not a room member.
    pub fn publish_track(&mut self, participant_id: Uuid, track: MediaTrack, payload: Vec<u8>) {
        if self.participants.contains(&participant_id) {
            self.published_tracks
                .insert((participant_id, track), payload);
        }
    }

    /// Subscribe a participant to a media track.
    ///
    /// Subscriptions are ignored when the participant is not a room member.
    pub fn subscribe_track(&mut self, participant_id: Uuid, track: MediaTrack) {
        if self.participants.contains(&participant_id) {
            self.subscriptions
                .entry(participant_id)
                .or_default()
                .insert(track);
        }
    }

    /// Get the most recent payload published by a participant for the given track.
    pub fn latest_payload(&self, participant_id: Uuid, track: MediaTrack) -> Option<Vec<u8>> {
        self.published_tracks.get(&(participant_id, track)).cloned()
    }

    /// Check whether a participant is subscribed to a media track.
    pub fn is_subscribed(&self, participant_id: Uuid, track: MediaTrack) -> bool {
        self.subscriptions
            .get(&participant_id)
            .is_some_and(|tracks| tracks.contains(&track))
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{MediaTrack, SfuConfig, SfuRoom};

    #[test]
    fn join_and_leave_room_returns_participant_id() {
        let config = SfuConfig {
            max_participants: 2,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        };

        let mut room = SfuRoom::new(config);
        let participant_id = room.join_room();

        assert!(room.participants().contains(&participant_id));

        let left_id = room.leave_room(participant_id);

        assert_eq!(left_id, participant_id);
        assert!(!room.participants().contains(&participant_id));
    }

    #[test]
    fn publish_and_subscribe_track_updates_state() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 4,
            video_codec: "vp8".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        let publisher = room.join_room();
        let subscriber = room.join_room();
        let payload = vec![1_u8, 2_u8, 3_u8];

        room.publish_track(publisher, MediaTrack::Video, payload.clone());
        room.subscribe_track(subscriber, MediaTrack::Video);

        assert_eq!(
            room.latest_payload(publisher, MediaTrack::Video),
            Some(payload)
        );
        assert!(room.is_subscribed(subscriber, MediaTrack::Video));
    }

    #[test]
    fn join_room_generates_unique_ids() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 10,
            video_codec: "h264".to_owned(),
            audio_codec: "aac".to_owned(),
        });

        let id_a: Uuid = room.join_room();
        let id_b: Uuid = room.join_room();

        assert_ne!(id_a, id_b);
    }

    #[test]
    fn leave_room_cleans_up_tracks_and_subscriptions() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 4,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        let participant = room.join_room();
        room.publish_track(participant, MediaTrack::Audio, vec![7, 8, 9]);
        room.subscribe_track(participant, MediaTrack::ScreenShare);
        assert!(room
            .latest_payload(participant, MediaTrack::Audio)
            .is_some());
        assert!(room.is_subscribed(participant, MediaTrack::ScreenShare));

        room.leave_room(participant);

        assert!(room
            .latest_payload(participant, MediaTrack::Audio)
            .is_none());
        assert!(!room.is_subscribed(participant, MediaTrack::ScreenShare));
    }

    #[test]
    fn unknown_participant_publish_and_subscribe_are_ignored() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 3,
            video_codec: "vp8".to_owned(),
            audio_codec: "opus".to_owned(),
        });
        let unknown = Uuid::new_v4();

        room.publish_track(unknown, MediaTrack::Video, vec![1, 2, 3]);
        room.subscribe_track(unknown, MediaTrack::Video);

        assert!(room.latest_payload(unknown, MediaTrack::Video).is_none());
        assert!(!room.is_subscribed(unknown, MediaTrack::Video));
    }

    #[test]
    fn publish_track_replaces_existing_payload_for_same_track() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 2,
            video_codec: "h264".to_owned(),
            audio_codec: "aac".to_owned(),
        });
        let publisher = room.join_room();

        room.publish_track(publisher, MediaTrack::Video, vec![1, 1, 1]);
        room.publish_track(publisher, MediaTrack::Video, vec![9, 9, 9]);

        assert_eq!(
            room.latest_payload(publisher, MediaTrack::Video),
            Some(vec![9, 9, 9])
        );
    }

    #[test]
    fn try_join_room_succeeds_when_under_capacity() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 2,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        let result = room.try_join_room();
        assert!(result.is_ok());
        assert!(room.participants().contains(&result.unwrap()));
    }

    #[test]
    fn try_join_room_fails_when_at_capacity() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 2,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        // Fill to capacity
        room.join_room();
        room.join_room();

        // Third join should fail
        let result = room.try_join_room();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::error::MeetingError::RoomCapacityExceeded {
                max_participants: 2
            }
        ));
    }

    // ============== Capacity Check Tests ==============

    #[test]
    fn try_join_room_capacity_boundary() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 3,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        // Join exactly at capacity
        let r1 = room.try_join_room();
        let r2 = room.try_join_room();
        let r3 = room.try_join_room();

        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());
        assert_eq!(room.participants().len(), 3);

        // One over capacity should fail
        let r4 = room.try_join_room();
        assert!(r4.is_err());
    }

    #[test]
    fn try_join_room_after_leave() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 2,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        let id1 = room.try_join_room().unwrap();
        let _id2 = room.try_join_room().unwrap();

        // At capacity
        let r3 = room.try_join_room();
        assert!(r3.is_err());

        // Leave one
        room.leave_room(id1);

        // Should be able to join again
        let r4 = room.try_join_room();
        assert!(r4.is_ok());
    }

    #[test]
    fn max_participants_one() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 1,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        let r1 = room.try_join_room();
        assert!(r1.is_ok());

        let r2 = room.try_join_room();
        assert!(r2.is_err());
    }

    #[test]
    fn max_participants_large_capacity() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 1000,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        // Add many participants
        for _ in 0..500 {
            let result = room.try_join_room();
            assert!(result.is_ok());
        }

        assert_eq!(room.participants().len(), 500);
    }

    #[test]
    fn join_room_vs_try_join_room_consistency() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 5,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        // Both should work under capacity
        let id1 = room.join_room();
        let id2 = room.try_join_room().unwrap();

        assert!(room.participants().contains(&id1));
        assert!(room.participants().contains(&id2));
    }

    #[test]
    fn capacity_with_tracks() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 2,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        let p1 = room.join_room();
        let p2 = room.join_room();

        // Publish and subscribe
        room.publish_track(p1, MediaTrack::Audio, vec![1, 2, 3]);
        room.publish_track(p1, MediaTrack::Video, vec![4, 5, 6]);
        room.subscribe_track(p2, MediaTrack::Audio);
        room.subscribe_track(p2, MediaTrack::Video);

        // At capacity - should still fail
        let r3 = room.try_join_room();
        assert!(r3.is_err());
    }

    #[test]
    fn leave_room_frees_capacity() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 1,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        let id = room.join_room();
        assert!(room.try_join_room().is_err());

        room.leave_room(id);
        assert!(room.try_join_room().is_ok());
    }

    #[test]
    fn capacity_error_message_contains_limit() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 1,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        room.join_room();
        let result = room.try_join_room();

        if let Err(crate::error::MeetingError::RoomCapacityExceeded { max_participants }) = result
        {
            assert_eq!(max_participants, 1);
        } else {
            panic!("Expected RoomCapacityExceeded error");
        }
    }

    #[test]
    fn rapid_join_leave_cycles() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 2,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        });

        for _ in 0..10 {
            let id1 = room.join_room();
            let id2 = room.join_room();

            assert!(room.try_join_room().is_err());

            room.leave_room(id1);
            room.leave_room(id2);

            assert_eq!(room.participants().len(), 0);
        }
    }

    #[test]
    fn config_reflects_capacity() {
        let config = SfuConfig {
            max_participants: 42,
            video_codec: "vp9".to_owned(),
            audio_codec: "opus".to_owned(),
        };

        let room = SfuRoom::new(config);
        assert_eq!(room.config().max_participants, 42);
    }
}
