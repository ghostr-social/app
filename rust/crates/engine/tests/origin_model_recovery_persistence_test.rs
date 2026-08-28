use crate::host_stats::HostStats;
use crate::origin_model::{
    Admission, AdmissionClaimTerminal, DecisionMode, ErrorReason, MediaClass,
    OriginAdmissionIntent, OriginContext, OriginModel, OriginObservation, OriginQuery,
    RequestMethod,
};

#[test]
fn loaded_origin_model_releases_an_inflight_recovery_trial() {
    let full = query(RequestMethod::FullGet, 900_000);
    let mut stats = HostStats::new();
    open_circuit(stats.origin_model_mut(), &full);
    let probe = stats
        .origin_model_mut()
        .claim(
            &full,
            5_000,
            DecisionMode::Normal,
            OriginAdmissionIntent::Delivery,
        )
        .into_parts()
        .1
        .expect("sparse probe claim");
    let sparse = OriginObservation::success(query(RequestMethod::RangeGet, 65_536), 5_100);
    stats
        .origin_model_mut()
        .complete_claim(probe, AdmissionClaimTerminal::Observed(&sparse));
    stats.origin_model_mut().claim(
        &full,
        5_101,
        DecisionMode::Normal,
        OriginAdmissionIntent::Delivery,
    );

    let loaded = HostStats::from_json(&stats.to_json()).expect("valid model snapshot");

    assert_eq!(
        loaded.origin_model().circuit_admission(&full, 5_102),
        Admission::RecoveryTrial
    );
}

fn query(method: RequestMethod, bytes: u64) -> OriginQuery {
    OriginQuery::new(
        "https://persistent.example/video.mp4",
        OriginContext::new(method, bytes, MediaClass::WholeObject),
    )
}

fn open_circuit(model: &mut OriginModel, query: &OriginQuery) {
    for at_ms in 1_000..=1_002 {
        model.observe(&OriginObservation::failure(
            query.clone(),
            at_ms,
            ErrorReason::Timeout,
        ));
    }
}
