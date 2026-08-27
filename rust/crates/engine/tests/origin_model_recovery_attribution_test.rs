use super::origin_model_recovery_fixture::{claim, open_circuit, query};
use crate::origin_model::{
    Admission, AdmissionClaimTerminal, DecisionMode, OriginModel, OriginObservation, OriginQuery,
    RequestMethod,
};

#[test]
fn sparse_full_recovery_trains_only_the_physical_range_profile() {
    let full = query(RequestMethod::FullGet, 900_000);
    let range = query(RequestMethod::RangeGet, 65_536);
    let mut model = open_circuit(&full);
    let active = claim(
        &mut model,
        &full,
        5_000,
        Admission::RecoveryProbe {
            maximum_bytes: 65_536,
        },
    );
    let full_before = samples(&model, &full);
    let range_before = samples(&model, &range);
    let physical = OriginObservation::success(range.clone(), 5_100);

    model.complete_claim(active, AdmissionClaimTerminal::Observed(&physical));

    assert_eq!(samples(&model, &full), full_before);
    assert!(samples(&model, &range) > range_before);
}

fn samples(model: &OriginModel, query: &OriginQuery) -> f64 {
    model
        .estimate(query, 5_101, DecisionMode::Normal)
        .effective_samples
}
