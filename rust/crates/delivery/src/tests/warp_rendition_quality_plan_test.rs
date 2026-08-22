use crate::delivery_events::DeliveryCandidate;
use crate::manager::state::DeliveryState;
use crate::tests::adaptive_plan_runner::{run, PlanScenario};
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::HashMap;

#[test]
fn production_plan_uses_only_complete_explicit_rendition_quality() {
    assert_eq!(quality_gain(Some(6_000_000)), 166_666);
    assert_eq!(quality_gain(None), 0);
}

fn quality_gain(ceiling_bitrate: Option<u64>) -> u64 {
    let post = PostId::new("quality");
    let low = rendition("low", Some(1_000_000));
    let high = rendition("high", ceiling_bitrate);
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: low.meta().clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![low, high],
        discovered_at: 1,
    });
    state.catalog_mut().learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    let work = run(PlanScenario {
        state,
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 4,
    });
    work.warp
        .expect("WARP decision")
        .generated
        .actions
        .into_iter()
        .find(|action| action.node.post == post && action.node.forecast.ready_playback_ms > 0)
        .expect("playable action")
        .node
        .forecast
        .quality_gain_micros
}

fn rendition(name: &str, bitrate: Option<u64>) -> VideoRendition {
    VideoRendition::try_new(
        VideoMeta {
            urls: vec![format!("https://media.example/{name}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: Some(format!("{name}-digest")),
            size_bytes: Some(1_000_000),
            duration_ms: Some(8_000),
        },
        bitrate,
    )
    .unwrap()
}
