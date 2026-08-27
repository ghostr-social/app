use super::query;
use crate::tests::support::planned_transfer;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginAttemptProfile, OriginContext, OriginModel,
    OriginObservation, OriginQuery, OriginRequestProfile, RequestMethod,
};

#[test]
fn admission_uses_the_planners_request_profile_without_rederiving_it() {
    let mut transfer = planned_transfer("prefix", "media.example", PreemptionAuthority::Transition);
    transfer.profile = OriginAttemptProfile::new(OriginRequestProfile::new(
        RequestMethod::PrefixGet,
        4,
        MediaClass::Unknown,
    ));
    let observed_at_ms = 10_000;
    let actual = query(&transfer, observed_at_ms, 1, NetworkClass::Wifi);
    let expected = OriginQuery::new(
        transfer.url.clone(),
        OriginContext::new(RequestMethod::PrefixGet, 4, MediaClass::Unknown)
            .with_network(NetworkClass::Wifi)
            .with_concurrency(1)
            .with_observed_at_ms(observed_at_ms),
    );
    let mut model = OriginModel::default();
    model.observe(&OriginObservation::success(actual, observed_at_ms));

    assert!(
        model
            .estimate(&expected, observed_at_ms, DecisionMode::Normal)
            .effective_samples
            > 0.9
    );
}
