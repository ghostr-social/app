use crate::delivery_events::{DeliveryCandidate, DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

const DISCOVERED: usize = 96;
const RETAINED: usize = 64;

#[test]
fn long_discovery_keeps_newest_candidates_without_bounding_the_focus_window() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    for index in 0..DISCOVERED {
        state.apply_candidate(candidate(index));
    }
    state.apply_focus(DeliveryFocus::compatibility(
        vec![focus_item("focus-only")],
        0,
        0,
    ));

    let posts = state.candidate_posts();
    assert_eq!(posts.len(), RETAINED + 1);
    assert_eq!(posts.first(), Some(&PostId::new("focus-only")));
    assert!(contains(&posts, DISCOVERED - 1));
    assert!(contains(&posts, DISCOVERED - RETAINED));
    assert!(!contains(&posts, DISCOVERED - RETAINED - 1));
    assert!(state.catalog().lookup(&PostId::new("focus-only")).is_some());
    for index in 0..DISCOVERED - RETAINED {
        assert!(
            state
                .catalog()
                .lookup(&PostId::new(candidate_id(index)))
                .is_none(),
            "old catalog entry {index}"
        );
    }
    for index in DISCOVERED - RETAINED..DISCOVERED {
        assert!(state
            .catalog()
            .lookup(&PostId::new(candidate_id(index)))
            .is_some());
    }
    assert_eq!(state.take_representation_bindings().len(), RETAINED + 1);
}

fn candidate(index: usize) -> DeliveryCandidate {
    let id = candidate_id(index);
    DeliveryCandidate {
        post: PostId::new(&id),
        meta: metadata(&id),
        renditions: Vec::new(),
        discovered_at: index as u64,
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

fn contains(posts: &[PostId], index: usize) -> bool {
    posts.contains(&PostId::new(candidate_id(index)))
}

fn candidate_id(index: usize) -> String {
    format!("candidate-{index:03}")
}
