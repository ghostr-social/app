use crate::delivery_events::{DeliveryCandidate, DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn probe_candidates_include_the_whole_upcoming_policy_frontier() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    assert!(state.probe_posts().is_empty());

    state.apply_candidate(candidate("first", 1));
    state.apply_candidate(candidate("latest", 2));
    assert_eq!(state.probe_posts(), vec![PostId::new("latest")]);

    state.apply_focus(
        DeliveryFocus::compatibility(
            [
                "behind", "current", "next-1", "next-2", "next-3", "far-1", "far-2",
            ]
            .into_iter()
            .map(focus_item)
            .collect(),
            1,
            0,
        ),
        0,
    );
    assert_eq!(
        state.probe_posts(),
        ["current", "next-1", "next-2", "next-3", "far-1", "far-2"].map(PostId::new)
    );
}

fn candidate(id: &str, discovered_at: u64) -> DeliveryCandidate {
    DeliveryCandidate {
        post: PostId::new(id),
        meta: metadata(id),
        renditions: Vec::new(),
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
        size_bytes: None,
        duration_ms: None,
    }
}
