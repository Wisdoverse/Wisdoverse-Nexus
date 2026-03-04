#[cfg(test)]
mod collaboration_test {
    use chrono::{Duration, Utc};
    use nexis_calendar::{detect_overlap, CalendarEvent, EventAttendee, ResponseStatus, TimeRange};
    use nexis_doc::{CRDTDocument, DocMetadata, DocVersion, Document};
    use nexis_meeting::{DefaultMediaRouter, MediaRouter, MediaTrack, SfuConfig, SfuRoom};
    use nexis_task::{
        DefaultTaskWorkflow, Task, TaskPriority, TaskSource, TaskStatus, TaskWorkflow,
    };
    use std::collections::HashSet;
    use uuid::Uuid;

    fn make_document(title: &str, content: &str) -> Document {
        let now = Utc::now();
        let author_id = Uuid::new_v4();

        Document {
            metadata: DocMetadata {
                id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                title: title.to_string(),
                created_by: author_id,
                created_at: now,
                updated_at: now,
            },
            content: content.to_string(),
            current_version: DocVersion {
                version: 1,
                created_at: now,
                created_by: author_id,
                checksum: None,
            },
            crdt_doc: Some(CRDTDocument::new()),
        }
    }

    fn make_task() -> Task {
        let now = Utc::now();
        Task {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            title: "Follow up with design team".to_string(),
            description: Some("Convert meeting notes into next sprint tasks".to_string()),
            status: TaskStatus::Created,
            assigned_to: None,
            block_reason: None,
            priority: TaskPriority::High,
            source: TaskSource::MeetingActionItem,
            due_at: Some(now + Duration::days(2)),
            created_at: now,
            updated_at: now,
        }
    }

    fn make_event(
        owner_id: Uuid,
        title: &str,
        start_offset_minutes: i64,
        duration_minutes: i64,
    ) -> CalendarEvent {
        let base = Utc::now();
        CalendarEvent {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            owner_id,
            title: title.to_string(),
            start_at: base + Duration::minutes(start_offset_minutes),
            end_at: base + Duration::minutes(start_offset_minutes + duration_minutes),
            attendees: vec![EventAttendee {
                member_id: owner_id,
                display_name: "Owner".to_string(),
                response_status: ResponseStatus::Accepted,
                optional: false,
            }],
            source_type: Some("meeting".to_string()),
            source_ref_id: None,
        }
    }

    fn make_recurring_events(
        owner_id: Uuid,
        title: &str,
        occurrences: usize,
        start_offset_days: i64,
        duration_minutes: i64,
    ) -> Vec<CalendarEvent> {
        let base = Utc::now();
        (0..occurrences)
            .map(|idx| {
                let start = base + Duration::days(start_offset_days + (idx as i64 * 7));
                CalendarEvent {
                    id: Uuid::new_v4(),
                    tenant_id: Uuid::new_v4(),
                    owner_id,
                    title: title.to_string(),
                    start_at: start,
                    end_at: start + Duration::minutes(duration_minutes),
                    attendees: vec![EventAttendee {
                        member_id: owner_id,
                        display_name: "Owner".to_string(),
                        response_status: ResponseStatus::Accepted,
                        optional: false,
                    }],
                    source_type: Some("recurring".to_string()),
                    source_ref_id: None,
                }
            })
            .collect()
    }

    #[test]
    fn test_meeting_room_lifecycle() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 8,
            video_codec: "vp9".to_string(),
            audio_codec: "opus".to_string(),
        });

        let publisher = room.join_room();
        let subscriber = room.join_room();

        assert_eq!(room.participants().len(), 2);

        let payload = vec![10_u8, 20_u8, 30_u8];
        room.publish_track(publisher, MediaTrack::Video, payload.clone());
        room.subscribe_track(subscriber, MediaTrack::Video);

        assert_eq!(
            room.latest_payload(publisher, MediaTrack::Video),
            Some(payload)
        );
        assert!(room.is_subscribed(subscriber, MediaTrack::Video));

        room.leave_room(publisher);

        assert!(!room.participants().contains(&publisher));
        assert_eq!(room.latest_payload(publisher, MediaTrack::Video), None);
        assert!(room.participants().contains(&subscriber));
    }

    #[test]
    fn test_sfu_media_routing_with_multiple_participants() {
        let mut room = SfuRoom::new(SfuConfig {
            max_participants: 10,
            video_codec: "vp9".to_string(),
            audio_codec: "opus".to_string(),
        });

        let sender = room.join_room();
        let member_a = room.join_room();
        let member_b = room.join_room();
        let member_c = room.join_room();

        room.subscribe_track(member_a, MediaTrack::Audio);
        room.subscribe_track(member_b, MediaTrack::Video);
        room.subscribe_track(member_c, MediaTrack::ScreenShare);

        let participants = room.participants().iter().copied().collect::<Vec<_>>();
        let recipients = DefaultMediaRouter::route_media(sender, MediaTrack::Video, &participants);

        let recipient_set = recipients.iter().copied().collect::<HashSet<_>>();
        assert_eq!(recipient_set.len(), 3);
        assert!(recipient_set.contains(&member_a));
        assert!(recipient_set.contains(&member_b));
        assert!(recipient_set.contains(&member_c));
        assert!(!recipient_set.contains(&sender));

        let payload = vec![9_u8, 8_u8, 7_u8];
        room.publish_track(sender, MediaTrack::Video, payload.clone());
        assert_eq!(
            room.latest_payload(sender, MediaTrack::Video),
            Some(payload)
        );
    }

    #[test]
    fn test_document_crdt_sync() {
        let mut document = make_document("Sprint notes", "Initial draft");

        assert_eq!(document.content, "Initial draft");

        document.content.push_str(" + action items");
        document.metadata.updated_at = Utc::now();
        document.current_version.version += 1;

        let crdt_update = document
            .crdt_doc
            .as_ref()
            .expect("CRDT document should exist")
            .encode_update();

        let replica = CRDTDocument::new();
        replica
            .apply_update(&crdt_update)
            .expect("CRDT update should apply");

        let current_content = document.content.clone();
        assert_eq!(current_content, "Initial draft + action items");
        assert_eq!(document.current_version.version, 2);
        assert_eq!(
            replica.get_content(),
            document
                .crdt_doc
                .as_ref()
                .expect("CRDT document should exist")
                .get_content()
        );
    }

    #[test]
    fn test_crdt_concurrent_sync_updates_last_write_visible() {
        let mut sync = nexis_doc::InMemorySyncProvider::new();
        let doc_id = Uuid::new_v4();

        // Simulate concurrent updates from two clients for the same doc.
        let update_from_client_a = vec![1_u8, 2_u8, 3_u8];
        let update_from_client_b = vec![4_u8, 5_u8, 6_u8, 7_u8];

        nexis_doc::DocumentSync::push_update(&mut sync, doc_id, update_from_client_a);
        nexis_doc::DocumentSync::push_update(&mut sync, doc_id, update_from_client_b.clone());

        let pulled = nexis_doc::DocumentSync::pull_update(&sync, doc_id)
            .expect("latest CRDT update should be available");
        assert_eq!(pulled, update_from_client_b);

        let state = nexis_doc::DocumentSync::state(&sync, doc_id)
            .expect("sync state should be tracked for document");
        assert_eq!(state.last_update_len, 4);
    }

    #[test]
    fn test_task_workflow() {
        let mut task = make_task();
        let assignee_id = Uuid::new_v4();

        let workflow = DefaultTaskWorkflow;
        let transition = workflow
            .transition(TaskStatus::Created, TaskStatus::Assigned)
            .expect("created -> assigned should be valid");
        assert_eq!(transition.from, TaskStatus::Created);
        assert_eq!(transition.to, TaskStatus::Assigned);

        task.assign_to(assignee_id)
            .expect("task assignment should succeed");
        task.start().expect("task start should succeed");
        task.complete().expect("task completion should succeed");

        assert_eq!(task.assigned_to, Some(assignee_id));
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.block_reason.is_none());
    }

    #[test]
    fn test_task_workflow_blocked_to_in_progress_edge_case() {
        let mut task = make_task();
        task.assign_to(Uuid::new_v4())
            .expect("created -> assigned should succeed");
        task.start()
            .expect("assigned -> in_progress should succeed");
        task.block("Waiting on legal review")
            .expect("in_progress -> blocked should succeed");

        assert_eq!(task.status, TaskStatus::Blocked);
        assert_eq!(
            task.block_reason.as_deref(),
            Some("Waiting on legal review")
        );

        let resumed = task
            .start()
            .expect("blocked -> in_progress should be allowed");
        assert_eq!(resumed.from, TaskStatus::Blocked);
        assert_eq!(resumed.to, TaskStatus::InProgress);
        assert!(resumed.side_effects.is_empty());

        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.block_reason.is_none());
    }

    #[test]
    fn test_calendar_conflict_detection() {
        let owner_id = Uuid::new_v4();
        let event_a = make_event(owner_id, "Design Sync", 0, 60);
        let event_b = make_event(owner_id, "Product Review", 30, 45);

        assert!(event_a.overlaps_with(&event_b));

        let overlap = detect_overlap(
            TimeRange::new(event_a.start_at, event_a.end_at),
            TimeRange::new(event_b.start_at, event_b.end_at),
        )
        .expect("events should overlap");

        assert_eq!(overlap.start, event_b.start_at);
        assert_eq!(overlap.end, event_a.end_at);
    }

    #[test]
    fn test_calendar_recurring_event_conflicts_detect_specific_occurrence() {
        let owner_id = Uuid::new_v4();
        let recurring = make_recurring_events(owner_id, "Weekly Planning", 4, 1, 60);

        // Recurring weekly events should not overlap each other.
        for window in recurring.windows(2) {
            let a = &window[0];
            let b = &window[1];
            assert!(!a.overlaps_with(b));
        }

        // Create a conflicting event on top of the third occurrence.
        let third = &recurring[2];
        let conflict = CalendarEvent {
            id: Uuid::new_v4(),
            tenant_id: third.tenant_id,
            owner_id,
            title: "Escalation Review".to_string(),
            start_at: third.start_at + Duration::minutes(30),
            end_at: third.end_at + Duration::minutes(15),
            attendees: third.attendees.clone(),
            source_type: Some("meeting".to_string()),
            source_ref_id: None,
        };

        assert!(third.overlaps_with(&conflict));

        let overlap = detect_overlap(
            TimeRange::new(third.start_at, third.end_at),
            TimeRange::new(conflict.start_at, conflict.end_at),
        )
        .expect("conflicting recurring occurrence should have overlap");

        assert_eq!(overlap.start, conflict.start_at);
        assert_eq!(overlap.end, third.end_at);
    }
}
