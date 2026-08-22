use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::tests::adaptive_plan_fixture::{source, state};
use crate::tests::adaptive_plan_runner::{run_with_retry, PlanScenario};
use ghostr_engine::adaptive::{PlannerCommand, PlannerRetryAvailability, StorageSnapshot};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::collections::HashMap;

const MIRROR: &str = "https://mirror.example/p0.mp4";

#[test]
fn delivery_projects_exact_origin_cooldown_into_the_warp_input() {
    let mut state = state();
    let post = PostId::new("p0");
    let primary = source(0);
    state.catalog_mut().upsert(
        post.clone(),
        VideoMeta {
            urls: vec![primary.clone(), MIRROR.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1_000_000),
            duration_ms: Some(8_000),
        },
    );
    let mut retry = RetryBook::new(RetryPolicy::default());
    retry
        .cool_down_until(post.clone(), 2_000)
        .expect("cooldown");
    let work = run_with_retry(scenario(state), &retry);
    let decision = work.warp.expect("WARP decision");

    assert_eq!(network_sources(&decision, &post).count(), 0);
    assert_eq!(
        decision.retry_availability,
        vec![ghostr_engine::adaptive::PlannerRetryEvidence::new(
            post,
            PlannerRetryAvailability::Cooling {
                eligible_at_ms: 2_000,
            },
        )]
    );
}

fn scenario(state: crate::manager::state::DeliveryState) -> PlanScenario<'static> {
    PlanScenario {
        state,
        buffer_ms: 0,
        bytes_per_second: 1_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 4,
    }
}

fn network_sources<'a>(
    decision: &'a ghostr_engine::adaptive::WarpPlanningDecision,
    post: &'a PostId,
) -> impl Iterator<Item = &'a str> {
    decision
        .generated
        .actions
        .iter()
        .filter(move |action| &action.node.post == post)
        .filter_map(|action| match &action.command {
            PlannerCommand::ProbeHead { source, .. } => Some(source.as_str()),
            PlannerCommand::Transfer(transfer) => Some(transfer.source.as_str()),
            _ => None,
        })
}
