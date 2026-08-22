use crate::delivery_events::DeliveryCandidate;
use crate::manager::state::DeliveryState;
use crate::tests::adaptive_plan_runner::{run, PlanScenario};
use ghostr_engine::adaptive::{RetrievalRung, StorageSnapshot};
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::{
    DataUsageLevel, DeliveryKind, EngineParams, PostId, PreviewDescriptor, VideoMeta,
};
use std::collections::HashMap;

pub(super) const BLURHASH: &str = "LEHV6nWB2yk8pyo0adR*.7kCMdnj";

#[test]
fn inline_blurhash_adds_preview_to_the_production_retrieval_frontier() {
    let with_preview = plan(PreviewDescriptor::inline_blurhash(BLURHASH));
    let unavailable = plan(None);

    assert!(has_preview(&with_preview));
    assert!(!has_preview(&unavailable));
}

pub(super) fn plan(preview: Option<PreviewDescriptor>) -> crate::manager::plan::PlannedWork {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let post = PostId::new("preview");
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: VideoMeta {
            urls: vec!["https://media.example/preview.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1_000_000),
            duration_ms: Some(8_000),
        },
        preview,
        metadata_evidence: Vec::new(),
        renditions: Vec::new(),
        discovered_at: 1,
    });
    state.catalog_mut().learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    run(PlanScenario {
        state,
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 4,
    })
}

fn has_preview(work: &crate::manager::plan::PlannedWork) -> bool {
    work.warp
        .as_ref()
        .unwrap()
        .generated
        .ladders
        .iter()
        .any(|ladder| {
            ladder
                .frontier
                .plans()
                .iter()
                .any(|plan| plan.terminal == RetrievalRung::Preview)
        })
}
