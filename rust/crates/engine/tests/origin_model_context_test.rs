use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginModel, OriginObservation,
    OriginOutcome, OriginQuery, RequestMethod,
};

fn query(url: &str, method: RequestMethod, bytes: u64) -> OriginQuery {
    OriginQuery::new(
        url,
        OriginContext::new(method, bytes, MediaClass::ProgressiveMp4)
            .with_network(NetworkClass::Wifi)
            .with_concurrency(2)
            .with_observed_at_ms(43_200_000),
    )
}

#[test]
fn observations_are_method_context_and_url_specific_with_hierarchical_shrinkage() {
    let mut model = OriginModel::default();
    let learned = query(
        "https://media.example/a.mp4",
        RequestMethod::RangeGet,
        64 * 1024,
    );
    for at in 1..=8 {
        model.observe(
            OriginObservation::success(learned.clone(), at * 1_000)
                .with_range_compliance(true)
                .with_ttfb_ms(40)
                .with_throughput_bps(8_000_000),
        );
    }

    let exact = model.estimate(&learned, 9_000, DecisionMode::Normal);
    let sibling = model.estimate(
        &query(
            "https://media.example/b.mp4",
            RequestMethod::RangeGet,
            64 * 1024,
        ),
        9_000,
        DecisionMode::Normal,
    );
    let other_method = model.estimate(
        &query("https://media.example/a.mp4", RequestMethod::Head, 0),
        9_000,
        DecisionMode::Normal,
    );

    assert!(exact.effective_samples > sibling.effective_samples);
    assert!(sibling.success.mean > other_method.success.mean);
    assert!(exact.throughput_bps.p50 > sibling.throughput_bps.p50);
    assert_eq!(exact.context.network, NetworkClass::Wifi);
}

#[test]
fn failures_keep_discounted_error_reason_frequencies() {
    let mut model = OriginModel::default();
    let key = query(
        "https://bad.example/a.mp4",
        RequestMethod::FullGet,
        8_000_000,
    );
    model.observe(OriginObservation::failure(
        key.clone(),
        1_000,
        ghostr_engine::origin_model::ErrorReason::Timeout,
    ));
    model.observe(OriginObservation::failure(
        key.clone(),
        2_000,
        ghostr_engine::origin_model::ErrorReason::Http5xx,
    ));
    model.observe(OriginObservation {
        outcome: OriginOutcome::Failure(ghostr_engine::origin_model::ErrorReason::Timeout),
        ..OriginObservation::failure(
            key.clone(),
            3_000,
            ghostr_engine::origin_model::ErrorReason::Timeout,
        )
    });

    let estimate = model.estimate(&key, 3_100, DecisionMode::Normal);
    assert_eq!(
        estimate.most_likely_error(),
        Some(ghostr_engine::origin_model::ErrorReason::Timeout)
    );
}
