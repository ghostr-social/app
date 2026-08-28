use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, ControlMode, MediaLayout, PlannerCommand, PlannerContext,
};
use crate::origin_model::{
    AdmissionClaimTerminal, DecisionMode, MediaClass, OriginAdmissionIntent, OriginContext,
    OriginModel, OriginObservation, OriginQuery, RequestMethod,
};
use crate::tests::adaptive_support::snapshot;

#[test]
fn spent_optional_probe_budget_preserves_delivery_work() {
    let mut input = snapshot(1, 20_000_000, 8_000, 0);
    input.candidates[0].layout = MediaLayout::RequiresCompleteFile;
    let mut base = AdaptivePlayabilityPolicy.plan(&input);
    assert_eq!(base.mode, ControlMode::Normal);
    let origins = spent_origin_budget(&input);
    let context = PlannerContext::explicitly_unavailable(&input);

    let normal = WarpActionGenerator::generate(&input, &base, &origins, &context);
    assert!(has_transfer(&normal));

    base.mode = ControlMode::Safety;
    let safety = WarpActionGenerator::generate(&input, &base, &origins, &context);
    assert!(has_transfer(&safety));
}

#[test]
fn spent_optional_probe_budget_preserves_the_preferred_delivery_origin() {
    let mut input = snapshot(1, 20_000_000, 8_000, 0);
    input.candidates[0].layout = MediaLayout::RequiresCompleteFile;
    let preferred = input.candidates[0].origins[0].source.clone();
    let mut mirror = input.candidates[0].origins[0].clone();
    mirror.source = "https://mirror.example/video.mp4".into();
    input.candidates[0].origins.push(mirror);
    input.candidates[0].preferred_source = Some(preferred);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let origins = spent_origin_budget(&input);

    let generated = WarpActionGenerator::generate(
        &input,
        &base,
        &origins,
        &PlannerContext::explicitly_unavailable(&input),
    );

    let source = generated
        .actions
        .iter()
        .find_map(|action| match &action.command {
            PlannerCommand::Transfer(allocation) => Some(allocation.source.as_str()),
            _ => None,
        });
    assert_eq!(source, Some(input.candidates[0].origins[0].source.as_str()));
}

fn spent_origin_budget(input: &crate::adaptive::PlayabilitySnapshot) -> OriginModel {
    let candidate = &input.candidates[0];
    let source = &candidate.origins[0].source;
    let query = OriginQuery::new(
        source,
        OriginContext::new(RequestMethod::PrefixGet, 65_536, MediaClass::ProgressiveMp4),
    );
    let mut model = OriginModel::default();
    let (_, claim) = model
        .claim(
            &query,
            1_000,
            DecisionMode::Normal,
            OriginAdmissionIntent::OptionalExploration,
        )
        .into_parts();
    let observed = OriginObservation::success(query, 1_001);
    assert!(model.complete_claim(
        claim.expect("cold origin exploration claim"),
        AdmissionClaimTerminal::Observed(&observed),
    ));
    model
}

fn has_transfer(actions: &crate::adaptive::GeneratedActions) -> bool {
    actions
        .actions
        .iter()
        .any(|action| matches!(action.command, PlannerCommand::Transfer(_)))
}
