use super::{active, build_handoff_state, provisional_state, CURRENT, NEXT, THIRD};
use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use crate::tests::candidate_catalog_fixture::candidate;
use ghostr_engine::{ActionId, ByteRange, ChunkId, DataUsageLevel, PostId};

const CURRENT_SOURCE: &str = "https://current.example/current.mp4";
const PREFIX_BYTES: u64 = 65_536;
const SIZE_BYTES: u64 = 293_999;

pub(in crate::tests) fn cross_origin_handoff_state(
    level: DataUsageLevel,
) -> (DeliveryState, [ActiveAction; 2]) {
    build_handoff_state(level, Some(CURRENT_SOURCE))
}

pub(in crate::tests) fn acknowledged_full_roster_handoff_state(
) -> (DeliveryState, [ActiveAction; 3]) {
    acknowledged_full_roster_handoff_state_at(DataUsageLevel::Aggressive)
}

pub(super) fn acknowledged_full_roster_handoff_state_at(
    level: DataUsageLevel,
) -> (DeliveryState, [ActiveAction; 3]) {
    let mut state = provisional_state(level, Some(CURRENT_SOURCE), None);
    let active = [
        active(&state, THIRD, 1, 4_000),
        active(&state, NEXT, 2, 4_000),
        active_at_source(&state, CURRENT, CURRENT_SOURCE, 3),
    ];
    assert!(state.apply_focus(canonical_focus(1), 1_000));
    assert!(state.apply_focus(canonical_focus(2), 1_001));
    (state, active)
}

fn active_at_source(
    state: &DeliveryState,
    post: &str,
    source: &str,
    action: u64,
) -> ActiveAction {
    let post = PostId::new(post);
    let identity = state
        .catalog()
        .transfer_identity(&post, source)
        .expect("provisional representation");
    ActiveAction::range_with_action(
        ActionId::new(action),
        ChunkId {
            post,
            range: ByteRange::new(0, PREFIX_BYTES),
        },
        identity,
        4_000,
    )
}

fn canonical_focus(generation: u64) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![
            focus_item(NEXT, 2, None),
            focus_item(THIRD, 1, None),
            focus_item(CURRENT, 3, Some(CURRENT_SOURCE)),
        ],
        previews: Vec::new(),
        current_index: 2,
        watch_ms: 0,
        generation: FocusGeneration::try_new(generation).expect("positive generation"),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}

fn focus_item(post: &str, discovered_at: u64, source: Option<&str>) -> FocusItem {
    let mut item = candidate(post, discovered_at);
    item.meta.size_bytes = Some(SIZE_BYTES);
    if let Some(source) = source {
        item.meta.urls = vec![source.to_owned()];
    }
    FocusItem {
        post: item.post,
        meta: item.meta,
    }
}
