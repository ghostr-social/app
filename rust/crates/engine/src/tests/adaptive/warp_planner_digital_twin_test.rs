use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, BeamConfig, CompletionTimes, DigitalTwin,
    HardBudget, ResourceCost, TwinConfig, TwinEpochs, TwinSearchContext, TwinState, WarpSearch,
};
use crate::{ByteRange, PostId};

fn action(id: u16, completion: CompletionTimes) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("video"),
        ActionKind::FetchRange(ByteRange::new(0, 64_000)),
        ActionValue::from_net_micros(2_000_000),
    )
    .with_resources(ResourceCost::new(64_000, 64_000, 0, 1))
    .with_forecast(ActionForecast::new(completion, 9_500, 2_000))
    .with_origin("https://origin.example/media")
}

#[test]
fn beam_uses_simulated_tail_delay_to_rank_partial_plans() {
    let slow = action(1, CompletionTimes::new(1_500, 2_000, 3_000, 3_500));
    let fast = action(2, CompletionTimes::new(80, 120, 180, 220));
    let mut twin = DigitalTwin::new(TwinConfig::new(64, 9_500));
    let mut simulation = TwinSearchContext::new(
        &mut twin,
        TwinState::new(0, 4_000_000, 50, 2),
        TwinEpochs::new(1, 1, 1),
    );
    let selected = WarpSearch::new(BeamConfig::new(2, 16, 64, 10_000)).choose_first_simulated(
        &[slow, fast.clone()],
        HardBudget::unlimited(),
        &mut simulation,
    );

    assert_eq!(selected.action, Some(fast));
}

#[test]
fn twin_uses_common_particles_and_reports_tail_readiness() {
    let mut twin = DigitalTwin::new(TwinConfig::new(64, 9_500));
    let state = TwinState::new(500, 4_000_000, 50, 2);
    let epochs = TwinEpochs::new(1, 1, 1);
    let fast = twin.evaluate(
        &state,
        &[action(1, CompletionTimes::new(80, 120, 180, 200))],
        epochs,
    );
    let slow = twin.evaluate(
        &state,
        &[action(2, CompletionTimes::new(800, 1_200, 1_800, 2_000))],
        epochs,
    );

    assert_eq!(fast.common_random_seed, slow.common_random_seed);
    assert!(fast.p95_visible_delay_ms < slow.p95_visible_delay_ms);
    assert!(fast.on_time_probability_bps > slow.on_time_probability_bps);
}

#[test]
fn twin_cache_key_invalidates_on_model_evidence_or_budget_epoch() {
    let mut twin = DigitalTwin::new(TwinConfig::new(16, 9_500));
    let state = TwinState::new(500, 4_000_000, 50, 2);
    let plan = [action(1, CompletionTimes::new(80, 120, 180, 200))];
    let first = twin.evaluate(&state, &plan, TwinEpochs::new(1, 1, 1));
    let repeated = twin.evaluate(&state, &plan, TwinEpochs::new(1, 1, 1));
    assert_eq!(first, repeated);
    assert_eq!(twin.cache_entries(), 1);

    twin.evaluate(&state, &plan, TwinEpochs::new(2, 1, 1));
    twin.evaluate(&state, &plan, TwinEpochs::new(2, 2, 1));
    twin.evaluate(&state, &plan, TwinEpochs::new(2, 2, 2));
    assert_eq!(twin.cache_entries(), 4);
}

#[test]
fn watch_duration_particles_reduce_useful_coverage_during_rapid_swipes() {
    let plan = [action(1, CompletionTimes::new(80, 120, 180, 200))];
    let mut twin = DigitalTwin::new(TwinConfig::new(128, 9_500));
    let patient = TwinState::new(500, 4_000_000, 50, 2).with_swipe_rate(2);
    let rapid = TwinState::new(500, 4_000_000, 50, 2).with_swipe_rate(60);
    let epochs = TwinEpochs::new(1, 1, 1);
    let patient = twin.evaluate(&patient, &plan, epochs);
    let rapid = twin.evaluate(&rapid, &plan, epochs);
    assert!(patient.expected_ready_coverage_ms > rapid.expected_ready_coverage_ms);
}
