use crate::delivery_events::{DeliveryCandidate, DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn explicit_candidate_priority_replaces_the_current_focus() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_candidate(candidate("first", 20));
    state.apply_candidate(candidate("second", 10));
    state.apply_focus(DeliveryFocus {
        items: vec![focus_item("first")],
        current_index: 0,
        watch_ms: 0,
    });

    state.prioritize(PostId::new("second"));

    assert_eq!(state.focus().current(), Some(&PostId::new("second")));
    assert_eq!(state.candidate_posts().len(), 2);
}

#[test]
fn hls_candidates_stay_outside_the_progressive_queue() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let mut hls = candidate("stream", 20);
    hls.meta.delivery = DeliveryKind::Hls;

    state.apply_candidate(hls);

    assert!(state.candidate_posts().is_empty());
}

fn candidate(id: &str, discovered_at: u64) -> DeliveryCandidate {
    DeliveryCandidate {
        post: PostId::new(id),
        meta: metadata(id),
        discovered_at,
    }
}

fn focus_item(id: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(id),
        meta: metadata(id),
    }
}

fn metadata(id: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}
