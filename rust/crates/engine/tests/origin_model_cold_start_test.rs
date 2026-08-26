use crate::origin_model::{
    Availability, ColdStartPrior, ColdStartSelector, DecisionMode, DomainClass, HttpProtocol,
    MediaClass, NetworkClass, OriginContext, OriginEnvironment, OriginModel, OriginQuery,
    RequestMethod, TlsVersion,
};

#[test]
fn cold_start_uses_available_cohort_inputs_and_preserves_missingness() {
    let mut model = OriginModel::default();
    model.register_cold_start(
        ColdStartSelector::default()
            .with_domain_class(DomainClass::ObjectStorage)
            .with_protocol(HttpProtocol::Http2)
            .with_tls_version(TlsVersion::Tls13)
            .with_method(RequestMethod::RangeGet),
        ColdStartPrior::new(18.0, 2.0, 45, 6_000_000),
    );
    let environment = OriginEnvironment::unavailable()
        .with_domain_class(DomainClass::ObjectStorage)
        .with_protocol(HttpProtocol::Http2)
        .with_tls_version(TlsVersion::Tls13);
    let query = OriginQuery::new(
        "https://bucket.example/media.mp4",
        OriginContext::new(RequestMethod::RangeGet, 65_536, MediaClass::ProgressiveMp4)
            .with_network(NetworkClass::Unavailable),
    )
    .with_environment(environment);

    let estimate = model.estimate(&query, 1_000, DecisionMode::Normal);
    assert!(estimate.success.mean > 0.85);
    assert_eq!(estimate.ttfb_ms.p50, 45);
    assert_eq!(estimate.environment.asn, Availability::Unavailable);
    assert_eq!(estimate.environment.region, Availability::Unavailable);
    assert_eq!(estimate.context.network, NetworkClass::Unavailable);
}
