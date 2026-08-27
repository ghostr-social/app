use super::{apply, query};
use crate::tests::support::planned_transfer;
use ghostr_engine::adaptive::{
    PreemptionAuthority, RetrievalRequest, WholeBodyContract, WholeFetchReason,
};
use ghostr_engine::origin_model::{
    Admission, DecisionMode, ErrorReason, MediaClass, NetworkClass, OriginAttemptContext,
    OriginAttemptProfile, OriginModel, OriginObservation, OriginQuery, OriginRequestProfile,
    RequestMethod,
};
use ghostr_engine::ByteRange;

#[test]
fn a_capped_request_keeps_forecast_identity_but_reports_executed_transport() {
    let mut transfer = planned_transfer("whole", "media.example", PreemptionAuthority::Transition);
    let forecast =
        OriginRequestProfile::new(RequestMethod::FullGet, 900_000, MediaClass::WholeObject);
    transfer.retrieval = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped {
            maximum_bytes: 900_000,
        },
        reason: WholeFetchReason::DirectCrossover,
    };
    transfer.profile = OriginAttemptProfile::new(forecast);

    let capped =
        apply(transfer, &Admission::RecoveryProbe { maximum_bytes: 4 }).expect("recovery request");

    assert_eq!(capped.profile.forecast(), forecast);
    assert_eq!(
        capped.profile.request(),
        OriginRequestProfile::new(RequestMethod::RangeGet, 4, MediaClass::WholeObject)
    );
}

#[test]
fn a_successful_prefix_recovery_closes_the_planners_circuit() {
    let mut transfer = planned_transfer("prefix", "media.example", PreemptionAuthority::Transition);
    let bytes = ByteRange::new(0, 65_536);
    let forecast = OriginRequestProfile::new(
        RequestMethod::PrefixGet,
        bytes.len(),
        MediaClass::ProgressiveMp4,
    );
    transfer.request.chunk.range = bytes;
    transfer.retrieval = RetrievalRequest::FetchRange {
        bytes,
        promotion: None,
    };
    transfer.profile = OriginAttemptProfile::new(forecast);

    let mut model = OriginModel::default();
    let planned_query = query(&transfer, 1, 1, NetworkClass::Wifi);
    for at_ms in 1..=3 {
        let failure =
            OriginObservation::failure(planned_query.clone(), at_ms, ErrorReason::Timeout);
        model.observe(&failure);
    }
    let admission = model.claim(&planned_query, 2_003, DecisionMode::Normal);
    assert!(matches!(admission, Admission::RecoveryProbe { .. }));
    let capped = apply(transfer, &admission).expect("recovery request");

    assert_eq!(capped.profile.forecast(), forecast);
    assert_eq!(capped.profile.request(), forecast);
    let context = OriginAttemptContext::new(capped.profile, NetworkClass::Wifi, 1, 2_003);
    let observed = OriginQuery::new(capped.url, context.request_context());
    model.observe(&OriginObservation::success(observed, 2_004));
    assert_eq!(
        model.circuit_admission(&planned_query, 2_004),
        Admission::Production
    );
}

#[test]
fn a_shortened_tail_cap_preserves_tail_identity_and_updates_bytes() {
    let mut transfer = planned_transfer("tail", "media.example", PreemptionAuthority::Transition);
    let forecast = OriginRequestProfile::new(RequestMethod::TailGet, 64, MediaClass::TailMoovRange);
    transfer.request.chunk.range = ByteRange::new(128, 192);
    transfer.retrieval = RetrievalRequest::FetchRange {
        bytes: transfer.request.chunk.range,
        promotion: None,
    };
    transfer.profile = OriginAttemptProfile::new(forecast);

    let capped =
        apply(transfer, &Admission::RecoveryProbe { maximum_bytes: 4 }).expect("tail probe");

    assert_eq!(capped.profile.forecast(), forecast);
    assert_eq!(
        capped.profile.request(),
        OriginRequestProfile::new(RequestMethod::TailGet, 4, MediaClass::TailMoovRange)
    );
}
