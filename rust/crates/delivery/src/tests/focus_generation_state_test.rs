use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn applied_focus_cannot_move_back_to_an_older_generation() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);

    assert!(state.apply_focus(focus("new", 2), 2));
    assert!(!state.apply_focus(focus("old", 1), 1));

    assert_eq!(state.focus().current(), Some(&PostId::new("new")));
}

#[test]
fn clearing_state_does_not_reopen_an_old_focus_generation() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    assert!(state.apply_focus(focus("new", 2), 2));

    state.clear();

    assert!(!state.apply_focus(focus("old", 1), 1));
    assert!(state.focus().current().is_none());
}

fn focus(post: &str, generation: u64) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new(post),
            meta: VideoMeta {
                urls: vec![format!("https://media.example/{post}.mp4")],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(16),
                duration_ms: Some(1_000),
            },
        }],
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).expect("positive generation"),
        transition: FocusTransition::UserNavigation,
    }
}
