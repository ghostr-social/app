use crate::adaptive::{
    AdaptivePlayabilityPolicy, BeamConfig, HeadProbeHistory, PlannerContext, PlannerQuality,
    PreviewAvailability, ViewProbability, WarpPlanner, WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::{ColdStartPrior, ColdStartSelector, OriginModel};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn ready_preview_reduces_marginal_quality_and_can_change_selected_post() {
    assert_eq!(selected(PreviewAvailability::Unavailable).as_str(), "p0");
    assert_eq!(
        selected(PreviewAvailability::Ready {
            bytes: 64,
            quality_micros: 900_000,
        })
        .as_str(),
        "p1"
    );
}

fn selected(preview: PreviewAvailability) -> PostId {
    let mut input = snapshot(2, 100_000_000, 20_000, 0);
    input.candidates.iter_mut().for_each(|candidate| {
        candidate.view_probability = ViewProbability::new(0.5).expect("valid test fixture");
        candidate.present.push(candidate.playable_ranges[0].bytes);
    });
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let p0 = input.candidates[0].post.clone();
    let p1 = input.candidates[1].post.clone();
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_quality(&p0, quality(900_000))
        .with_quality(&p1, quality(100_000))
        .with_preview(&p0, preview)
        .with_head_probe_history(&p0, HeadProbeHistory::Completed)
        .with_head_probe_history(&p1, HeadProbeHistory::Completed);
    let config = WarpPlannerConfig {
        beam: BeamConfig::new(1, 32, 256, u64::MAX),
        ..WarpPlannerConfig::default()
    };
    WarpPlanner::new(config)
        .plan(WarpPlannerInput::new(
            &input,
            &base,
            &reliable_origin(),
            &context,
        ))
        .selected
        .expect("selected action")
        .node
        .post
}

fn reliable_origin() -> OriginModel {
    let mut model = OriginModel::default();
    let mut prior = ColdStartPrior::new(100.0, 0.1, 1, 100_000_000);
    prior.range_alpha = 100.0;
    prior.range_beta = 0.1;
    model.register_cold_start(ColdStartSelector::default(), prior);
    model
}

const fn quality(expected_micros: u64) -> PlannerQuality {
    PlannerQuality::Estimated {
        expected_micros,
        lower_micros: expected_micros,
        uncertainty_bps: 0,
    }
}
