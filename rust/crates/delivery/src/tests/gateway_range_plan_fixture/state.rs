use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use crate::tests::media_timeline_fixture::install_classic_timeline;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

pub(super) fn state(post: &PostId, size_bytes: u64) -> DeliveryState {
    let meta = VideoMeta {
        urls: vec!["https://media.example/video.mp4".into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(size_bytes),
        duration_ms: Some(1_000),
    };
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let item = FocusItem {
        post: post.clone(),
        meta,
    };
    state.apply_focus(DeliveryFocus::compatibility(vec![item], 0, 0), 0);
    state.catalog_mut().learn(
        post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    install_classic_timeline(&mut state, post, 100, 100);
    state
}
