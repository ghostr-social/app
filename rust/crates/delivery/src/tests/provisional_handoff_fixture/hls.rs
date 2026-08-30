use super::{active, provisional_state, CURRENT, NEXT, THIRD};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, PostId, VideoMeta};

pub(in crate::tests) fn handoff_to_hls_state() -> (DeliveryState, [ActiveAction; 2]) {
    let mut state = provisional_state(DataUsageLevel::Conservative, None, None);
    let active = [
        active(&state, THIRD, 1, 4_000),
        active(&state, NEXT, 2, 4_000),
    ];
    assert!(state.apply_focus(hls_focus(), 1_000));
    (state, active)
}

fn hls_focus() -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new(CURRENT),
            meta: VideoMeta {
                urls: vec!["https://media.example/root.m3u8".into()],
                delivery: DeliveryKind::Hls,
                sha256: None,
                size_bytes: None,
                duration_ms: Some(4_000),
            },
        }],
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(1).expect("positive generation"),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}
