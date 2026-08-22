use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn timeline_window_starts_at_current_then_moves_forward_before_history() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility((0..8).map(focus_item).collect(), 3, 0),
        0,
    );

    assert_eq!(
        state.timeline_window_posts(),
        [3, 4, 5, 6, 7, 2, 1, 0].map(|index| PostId::new(format!("post-{index}")))
    );
}

fn focus_item(index: usize) -> FocusItem {
    FocusItem {
        post: PostId::new(format!("post-{index}")),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{index}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(32),
            duration_ms: Some(1_000),
        },
    }
}
