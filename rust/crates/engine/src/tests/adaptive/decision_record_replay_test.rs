use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionModelInput, DecisionPrivacy, DecisionRecord,
    DecisionRecordInput, DecisionReplayStatus, PrunedReason, ShadowPrices,
};
use crate::tests::adaptive_support::snapshot;

#[test]
fn a_private_decision_record_replays_the_exact_deterministic_plan() {
    let mut state = snapshot(4, 20_000_000, 8_000, 18);
    state.playback.current = crate::PostId::new("secret-current");
    state.candidates[0].post = state.playback.current.clone();
    state.candidates[0].origins[0].source = "https://private.example/a.mp4?token=raw".into();
    state.candidates[3].retrieval_eligible = false;
    let plan = AdaptivePlayabilityPolicy.plan(&state);
    let models: Vec<_> = (0..80)
        .map(|index| model(&format!("https://private.example/{index}?token=raw")))
        .collect();

    let privacy = DecisionPrivacy::from_key([9; 32]);
    let mut record = DecisionRecord::capture(DecisionRecordInput {
        sequence: 7,
        snapshot: &state,
        allocation: &plan,
        shadow_prices: ShadowPrices::new(12, 34, 0, 56),
        models: &models,
        privacy: &privacy,
    });
    let json = serde_json::to_string(&record).unwrap();
    let exported: DecisionRecord = serde_json::from_str(&json).unwrap();

    assert!(!json.contains("secret-current"));
    assert!(!json.contains("private.example"));
    assert!(!json.contains("token=raw"));
    assert!(!record.admissible_candidates.is_empty());
    assert!(!record.retained_plans.is_empty());
    assert!(!record.pruned.is_empty());
    assert_eq!(record.model_quantiles.len(), 64);
    assert_eq!(record.model_quantiles[0].ttfb_ms.p99, 900);
    assert_eq!(record.shadow_prices.network_micros, 12);
    assert!(record.chosen_action.is_some());
    assert_ne!(record.random_seed, 0);
    assert_eq!(record.replay(), DecisionReplayStatus::Verified);
    assert_eq!(exported.replay(), DecisionReplayStatus::Verified);
    assert!(record.bind_action(crate::ActionId::new(44)));
    assert!(!record.bind_action(crate::ActionId::new(45)));
    assert!(record.resolve(crate::adaptive::DecisionOutcome::Succeeded {
        bytes: 64,
        elapsed_ms: 20,
    }));
    assert!(!record.resolve(crate::adaptive::DecisionOutcome::Superseded));
}

#[test]
fn an_invalid_request_origin_is_recorded_as_unavailable() {
    let mut state = snapshot(1, 20_000_000, 8_000, 18);
    state.candidates[0].origins[0].source = "not a URL".into();
    let plan = AdaptivePlayabilityPolicy.plan(&state);
    let privacy = DecisionPrivacy::from_key([7; 32]);

    let record = DecisionRecord::capture(DecisionRecordInput {
        sequence: 1,
        snapshot: &state,
        allocation: &plan,
        shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &privacy,
    });

    assert!(record.admissible_candidates.is_empty(), "{record:#?}");
    assert_eq!(record.pruned[0].reasons, [PrunedReason::NoAvailableOrigin]);
}

fn model(source: &str) -> DecisionModelInput {
    DecisionModelInput {
        source: source.into(),
        success_bps: [9_000, 8_000, 9_900, 8_000],
        range_compliance_bps: [8_500, 7_000, 9_800, 7_000],
        ttfb_ms: [100, 400, 600, 700, 900, 700],
        throughput_bps: [
            4_000_000, 8_000_000, 10_000_000, 11_000_000, 12_000_000, 4_000_000,
        ],
        effective_samples: 12,
        adapting: true,
        uncertainty_bps: 1_200,
    }
}
