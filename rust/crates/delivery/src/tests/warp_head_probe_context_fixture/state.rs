use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

const CURRENT_SOURCE: &str = "https://fixture-current.example/video.mp4";

enum TargetPosition {
    Current,
    Ahead,
}

pub(in crate::tests) fn current_state(post: PostId, source: &str) -> DeliveryState {
    state_with_meta(post, vec![source.to_owned()], None, TargetPosition::Current)
}

pub(in crate::tests) fn ahead_state(post: PostId, source: &str) -> DeliveryState {
    ahead_state_with_sources(post, vec![source.to_owned()])
}

pub(in crate::tests) fn ahead_state_with_sources(
    post: PostId,
    sources: Vec<String>,
) -> DeliveryState {
    state_with_meta(post, sources, None, TargetPosition::Ahead)
}

pub(in crate::tests) fn ahead_state_with_size(
    post: PostId,
    source: &str,
    size: u64,
) -> DeliveryState {
    state_with_meta(
        post,
        vec![source.to_owned()],
        Some(size),
        TargetPosition::Ahead,
    )
}

fn state_with_meta(
    post: PostId,
    sources: Vec<String>,
    size_bytes: Option<u64>,
    position: TargetPosition,
) -> DeliveryState {
    let target = item(post, sources, size_bytes);
    let items = match position {
        TargetPosition::Current => vec![target],
        TargetPosition::Ahead => vec![
            item(current_post(), vec![CURRENT_SOURCE.into()], Some(1)),
            target,
        ],
    };
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(DeliveryFocus::compatibility(items, 0, 0), 0);
    state
}

fn current_post() -> PostId {
    PostId::new("fixture-current")
}

fn item(post: PostId, urls: Vec<String>, size_bytes: Option<u64>) -> FocusItem {
    FocusItem {
        post,
        meta: VideoMeta {
            urls,
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes,
            duration_ms: None,
        },
    }
}
