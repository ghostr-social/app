use crate::tests::{
    adaptive_plan_fixture,
    adaptive_plan_runner::{run, PlanScenario},
    media_timeline_fixture::classic_samples,
};
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use ghostr_engine::{ByteRange, PostId};
use std::collections::HashMap;

#[test]
fn continuation_deficit_protects_the_next_missing_dependency_before_decoder_blocking() {
    let mut state = adaptive_plan_fixture::state();
    let post = PostId::new("p0");
    let binding = state.catalog().binding(&post).expect("fixture");
    let offsets: Vec<u32> = (1..=24).map(|sample| sample * 100).collect();
    let mut movie = classic_samples(&offsets, &[100; 24]);
    // This fixture's legacy table helper has no handler; supply the video handler.
    add_video_handler(&mut movie);
    let prefix = [
        16_u32.to_be_bytes().as_slice(),
        b"ftypisom\0\0\0\0",
        9_984_u32.to_be_bytes().as_slice(),
        b"mdat",
    ]
    .concat();
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(10_000, &movie),
    ])
    .expect("fixture");
    assert!(timeline.startup_footprint().is_some());
    state.catalog_mut().learn_timeline_for(&binding, timeline);
    let present = HashMap::from([(
        post.clone(),
        vec![
            ByteRange::new(0, 300),
            ByteRange::new(10_000, 10_000 + movie.len() as u64),
        ],
    )]);
    let work = run(PlanScenario {
        state,
        buffer_ms: 2_000,
        bytes_per_second: 80,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present,
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 2,
    });
    let snapshot = work.snapshot.expect("fixture");
    let candidate = snapshot
        .candidates
        .iter()
        .find(|candidate| candidate.post == post)
        .expect("fixture");
    let needed = candidate
        .demanded
        .expect("protected continuation before an actual decoder read");
    assert_eq!(needed.start, 300);
    assert!(
        needed.end > 500,
        "protect more than the fixed startup cushion"
    );
}

fn add_video_handler(movie: &mut Vec<u8>) {
    let mut end = 0;
    for kind in [b"moov", b"trak", b"mdia"] {
        let at = movie
            .windows(4)
            .position(|value| value == kind)
            .expect("fixture")
            - 4;
        let size = u32::from_be_bytes(movie[at..at + 4].try_into().expect("fixture"));
        movie[at..at + 4].copy_from_slice(&(size + 24).to_be_bytes());
        end = at + size as usize;
    }
    let handler = [
        24_u32.to_be_bytes().as_slice(),
        b"hdlr\0\0\0\0\0\0\0\0vide\0\0\0\0",
    ]
    .concat();
    movie.splice(end..end, handler);
}
