use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn accepted_focus_changes_record_recent_forward_and_backward_navigation() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    assert!(state.apply_focus(focus(0), 1_000));
    assert!(state.apply_focus(focus(1), 2_000));
    assert!(state.apply_focus(focus(2), 3_000));
    assert!(state.apply_focus(focus(1), 4_000));

    let navigation = state.navigation(4_000);
    assert_eq!(navigation.forward_swipes_per_minute, 12);
    assert_eq!(navigation.backward_swipes_per_minute, 6);
}

fn focus(current_index: usize) -> DeliveryFocus {
    DeliveryFocus::compatibility((0..4).map(item).collect(), current_index, 0)
}

fn item(index: usize) -> FocusItem {
    FocusItem {
        post: PostId::new(format!("p{index}")),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/p{index}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1_000),
            duration_ms: Some(1_000),
        },
    }
}
