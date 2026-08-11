use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::plan::{planned_work, PlanInputs};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use crate::playback_demand::DemandSignal;
use crate::tests::media_timeline_fixture::classic_moov;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use ghostr_engine::tiers::DemandSignals;
use ghostr_engine::{ByteRange, DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::HashMap;

#[test]
fn gateway_demand_is_an_emergency_and_preserves_its_exact_range() {
    let post = PostId::new("current");
    let mut state = state(post.clone());
    let moov = classic_moov(100, 100);
    let timeline = parse_mp4_segments(&[MediaSegment::new(10_000, &moov)]).unwrap();
    let binding = state.catalog().binding(&post).unwrap();
    assert!(state.catalog_mut().learn_timeline_for(&binding, timeline));
    let present = HashMap::from([(
        post.clone(),
        vec![
            ByteRange::new(100, 200),
            ByteRange::new(10_000, 10_000 + moov.len() as u64),
        ],
    )]);
    let demanded = ByteRange::new(9_000, 9_100);
    let retry = RetryBook::new(RetryPolicy::default());

    let work = planned_work(
        &mut state,
        PlanInputs {
            stats: &HostStats::new(),
            retry: &retry,
            present: &present,
            demand: DemandSignals {
                gateway_demand: true,
                ..DemandSignals::default()
            },
            observed_at_ms: 1,
            demanded: Some(DemandSignal {
                post: post.clone(),
                range: demanded,
            }),
        },
    );

    assert!(work.emergency, "uncovered gateway demand is urgent");
    assert!(work
        .transfers
        .iter()
        .any(|transfer| transfer.request.chunk.range == demanded));
}

fn state(post: PostId) -> DeliveryState {
    let meta = VideoMeta {
        urls: vec!["https://media.example/video.mp4".into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(20_000),
        duration_ms: Some(1_000),
    };
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    assert!(state.apply_focus(DeliveryFocus::compatibility(
        vec![FocusItem { post, meta }],
        0,
        0,
    )));
    state
}
