use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

const ROSTER: usize = 200;
const CURRENT: usize = 50;

/// The UI hands over the complete feed roster; the per-event planning
/// pass must work on a bounded neighbourhood of the current post, not
/// scale store reads and snapshots with feed length.
#[test]
fn planning_window_is_bounded_around_the_current_post() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility(
            (0..ROSTER).map(|index| focus_item(&format!("post-{index}"))).collect(),
            CURRENT,
            0,
        ),
        0,
    );

    let planning = state.planning_window_posts();

    assert!(
        planning.len() < 40,
        "planning work grew with the roster: {} posts",
        planning.len()
    );
    assert!(planning.contains(&PostId::new(format!("post-{CURRENT}"))));
    assert!(planning.contains(&PostId::new(format!("post-{}", CURRENT + 1))));
    assert!(planning.contains(&PostId::new(format!("post-{}", CURRENT - 1))));
    assert_eq!(state.window_posts().len(), ROSTER);
}

fn focus_item(id: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(id),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    }
}
