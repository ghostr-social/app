use crate::tests::adaptive_plan_fixture::state;
use crate::tests::adaptive_plan_runner::{run, run_with_watch_model, PlanScenario};
use ghostr_engine::adaptive::{SemanticScore, StorageSnapshot};
use ghostr_engine::watch_model::{
    WatchContext, WatchKey, WatchModel, WatchNavigation, WatchSample, WatchSampleKind,
};
use ghostr_engine::PostId;
use std::collections::HashMap;

#[test]
fn production_plan_uses_learned_reach_semantic_deadline_and_model_epoch() {
    let cold = run(scenario());
    let model = trained_model();
    let learned = run_with_watch_model(scenario(), &model);
    let snapshot = learned.snapshot.as_ref().expect("snapshot");
    let decision = learned.warp.as_ref().expect("WARP decision");
    let p1 = PostId::new("p1");
    let expected = model.predict_window(&contexts(), 1_000);
    let reach = expected.candidates()[1].reach_probability();
    let candidate = snapshot
        .candidates
        .iter()
        .find(|candidate| candidate.post == p1)
        .unwrap();
    let evidence = decision.planner_candidate_evidence(&p1).unwrap();

    assert!((candidate.view_probability.value() - reach).abs() < f64::EPSILON);
    assert_eq!(
        evidence.semantic,
        SemanticScore::Known((reach * 1_000_000.0).round() as u64),
    );
    assert_eq!(evidence.watch.reach_probability_bps(), Some(bps(reach)));
    assert_eq!(
        decision.planner_epochs().unwrap().model,
        model.change_epoch()
    );
    assert!(model.change_epoch() > 0);
    assert_ne!(
        cold.warp.unwrap().common_random_seed,
        decision.common_random_seed,
    );
}

fn trained_model() -> WatchModel {
    let mut model = WatchModel::default();
    for index in 0..8 {
        model.observe(WatchSample::new(
            contexts()[0].clone(),
            500,
            WatchSampleKind::Abandoned,
            100 + index,
        ));
        model.observe_navigation(WatchNavigation::Forward, 100 + index);
    }
    model
}

fn contexts() -> Vec<WatchContext> {
    (0..12)
        .map(|index| WatchContext::new(WatchKey::digest(&format!("p{index}")), Some(8_000)))
        .collect()
}

fn scenario() -> PlanScenario<'static> {
    PlanScenario {
        state: state(),
        buffer_ms: 8_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 4,
    }
}

fn bps(value: f64) -> u16 {
    (value * 10_000.0).round() as u16
}
