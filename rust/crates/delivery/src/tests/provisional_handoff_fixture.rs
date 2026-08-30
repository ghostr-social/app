use crate::delivery_events::{DeliveryFocus, FocusGeneration, FocusItem, FocusTransition};
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use crate::tests::candidate_catalog_fixture::candidate;
use ghostr_engine::{ActionId, ByteRange, ChunkId, DataUsageLevel, EngineParams, PostId};

#[path = "provisional_handoff_fixture/hls.rs"]
mod hls;
#[path = "provisional_handoff_fixture/negative.rs"]
mod negative;
#[path = "provisional_handoff_fixture/provenance.rs"]
mod provenance;
#[path = "provisional_handoff_fixture/terminal.rs"]
pub(super) mod terminal;
pub(super) use hls::handoff_to_hls_state;
pub(super) use negative::{detached_next, handoff_with_expired_third, DetachedFuture, OBSERVED_AT_MS};
pub(super) use provenance::{acknowledged_full_roster_handoff_state, cross_origin_handoff_state};

pub(super) const CURRENT: &str = "current";
pub(super) const NEXT: &str = "next";
pub(super) const THIRD: &str = "third";
pub(super) fn handoff_state(level: DataUsageLevel) -> (DeliveryState, [ActiveAction; 2]) {
    build_handoff_state(level, None)
}

fn build_handoff_state(
    level: DataUsageLevel,
    current_source: Option<&str>,
) -> (DeliveryState, [ActiveAction; 2]) {
    let mut state = provisional_state(level, current_source, None);
    let active = [
        active(&state, THIRD, 1, 4_000),
        active(&state, NEXT, 2, 4_000),
    ];
    assert!(state.apply_focus(partial_canonical_focus(current_source), 1_000));
    (state, active)
}

fn provisional_state(
    level: DataUsageLevel,
    current_source: Option<&str>,
    next_digest: Option<&str>,
) -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), level);
    let mut items = [
        candidate(CURRENT, 3),
        candidate(NEXT, 2),
        candidate(THIRD, 1),
    ];
    if let Some(source) = current_source {
        items[0].meta.urls = vec![source.to_owned()];
    }
    items[1].meta.sha256 = next_digest.map(str::to_owned);
    for mut item in items {
        item.meta.size_bytes = Some(293_999);
        state.apply_candidate(item);
    }
    state
}

fn active(state: &DeliveryState, post: &str, action: u64, committed_until_ms: u64) -> ActiveAction {
    let source = format!("https://media.example/{post}.mp4");
    let post = PostId::new(post);
    let identity = state
        .catalog()
        .transfer_identity(&post, &source)
        .expect("provisional representation");
    ActiveAction::range_with_action(
        ActionId::new(action),
        ChunkId {
            post,
            range: ByteRange::new(0, 65_536),
        },
        identity,
        committed_until_ms,
    )
}

fn partial_canonical_focus(current_source: Option<&str>) -> DeliveryFocus {
    let mut current = candidate(CURRENT, 3);
    if let Some(source) = current_source {
        current.meta.urls = vec![source.to_owned()];
    }
    DeliveryFocus {
        items: vec![FocusItem {
            post: current.post,
            meta: current.meta,
        }],
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(1).expect("positive generation"),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}
