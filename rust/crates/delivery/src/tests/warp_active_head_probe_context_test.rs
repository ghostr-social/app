use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::plan::{planned_work, PlanInputs};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use crate::tests::support::transfer_identity;
use ghostr_engine::adaptive::{PlannerCommand, StorageSnapshot};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::{HashMap, HashSet};

#[test]
fn only_the_current_active_probe_identity_suppresses_head() {
    let post = PostId::new("post");
    let source = "https://media.example/video.mp4";
    let mut state = state(post.clone(), source);
    let current = state.catalog().transfer_identity(&post, source).unwrap();
    let stale = transfer_identity(&post, source);

    assert!(!generates_head(plan(&mut state, &[current], 2)));
    assert!(generates_head(plan(&mut state, &[stale], 2)));
}

#[test]
fn active_current_head_leaves_one_scoped_body_companion_slot() {
    let post = PostId::new("post");
    let source = "https://media.example/video.mp4";
    let mut state = state(post.clone(), source);
    let current = state.catalog().transfer_identity(&post, source).unwrap();
    let work = plan(&mut state, &[current], 1);
    let selected = work.warp.unwrap().selected.expect("body companion action");

    assert!(matches!(selected.command, PlannerCommand::Transfer(_)));
    assert_eq!(selected.node.post, post);
}

fn generates_head(work: crate::manager::plan::PlannedWork) -> bool {
    work.warp.unwrap().generated.actions.iter().any(|action| {
        matches!(&action.command, PlannerCommand::ProbeHead { post, .. } if post.as_str() == "post")
    })
}

fn plan(
    state: &mut DeliveryState,
    active: &[TransferIdentity],
    capacity: usize,
) -> crate::manager::plan::PlannedWork {
    planned_work(
        state,
        PlanInputs {
            stats: &HostStats::new(),
            retry: &RetryBook::new(RetryPolicy::default()),
            present: &HashMap::new(),
            finalized: &HashSet::new(),
            stored_totals: &HashMap::new(),
            continuation_sources: &HashMap::new(),
            revisions: &HashMap::new(),
            independent_sources: &HashMap::new(),
            completed_head_probes: &HashSet::new(),
            in_flight: &[],
            active_head_probes: active,
            storage: StorageSnapshot::new(1_000_000, 0),
            connection_capacity: capacity,
            connection_ceiling: 3,
            per_authority_request_limit: 3,
            packet_loss_bps: 0,
            measured_network_bytes_per_second: 0,
            capacity_revision: 0,
            observed_at_ms: 1,
            demanded: &HashMap::new(),
        },
    )
}

fn state(post: PostId, source: &str) -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility(
            vec![FocusItem {
                post,
                meta: VideoMeta {
                    urls: vec![source.to_owned()],
                    delivery: DeliveryKind::Progressive,
                    sha256: None,
                    size_bytes: None,
                    duration_ms: None,
                },
            }],
            0,
            0,
        ),
        0,
    );
    state
}
