use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use nexis_meeting::{MediaTrack, SfuConfig, SfuRoom};

fn benchmark_join_leave_room(c: &mut Criterion) {
    let config = SfuConfig {
        max_participants: 64,
        video_codec: "vp9".to_owned(),
        audio_codec: "opus".to_owned(),
    };

    c.bench_function("sfu/join_leave_room", |b| {
        b.iter_batched(
            || SfuRoom::new(config.clone()),
            |mut room| {
                let participant = room.join_room();
                room.leave_room(participant);
            },
            BatchSize::SmallInput,
        );
    });
}

fn benchmark_publish_subscribe_track(c: &mut Criterion) {
    let config = SfuConfig {
        max_participants: 64,
        video_codec: "vp9".to_owned(),
        audio_codec: "opus".to_owned(),
    };

    c.bench_function("sfu/publish_subscribe_track", |b| {
        b.iter_batched(
            || {
                let mut room = SfuRoom::new(config.clone());
                let publisher = room.join_room();
                let subscriber = room.join_room();
                (room, publisher, subscriber, vec![0_u8; 1024])
            },
            |(mut room, publisher, subscriber, payload)| {
                room.publish_track(publisher, MediaTrack::Video, payload);
                room.subscribe_track(subscriber, MediaTrack::Video);
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    sfu_benches,
    benchmark_join_leave_room,
    benchmark_publish_subscribe_track
);
criterion_main!(sfu_benches);
