
use crate::delivery_events::{DeliveryCandidate, DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn progressive_to_hls_removes_the_stale_progressive_representation() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let post = PostId::new("stream");
    assert!(state.apply_focus(focus(1, DeliveryKind::Progressive), 1));
    state.take_representation_bindings();
    assert!(state.catalog().lookup(&post).is_some());

    assert!(state.apply_focus(focus(2, DeliveryKind::Hls), 2));

    assert!(state.catalog().lookup(&post).is_none());
    assert_eq!(state.take_changed_representations(), vec![post]);
    assert!(state.take_representation_bindings().is_empty());
}

#[test]
fn late_progressive_candidate_cannot_override_canonical_hls_focus() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let post = PostId::new("stream");
    assert!(state.apply_focus(focus(1, DeliveryKind::Hls), 1));

    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: meta(DeliveryKind::Progressive),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: Vec::new(),
        discovered_at: 2,
    });

    assert!(state.catalog().lookup(&post).is_none());
    assert!(state.take_representation_bindings().is_empty());
}

fn focus(generation: u64, delivery: DeliveryKind) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("stream"),
            meta: meta(delivery),
        }],
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).expect("valid test fixture"),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}

fn meta(delivery: DeliveryKind) -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/video".to_owned()],
        delivery,
        sha256: None,
        size_bytes: None,
        duration_ms: Some(4_000),
    }
}
