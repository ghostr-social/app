use crate::host_stats::HostStats;
use crate::origin_model::{
    DecisionMode, MediaClass, OriginContext, OriginObservation, OriginQuery, RequestMethod,
};

#[test]
fn host_stats_snapshot_round_trips_the_origin_model() {
    let query = OriginQuery::new(
        "https://persistent.example/video.mp4",
        OriginContext::new(RequestMethod::FullGet, 900_000, MediaClass::WholeObject),
    );
    let mut stats = HostStats::new();
    stats.origin_model_mut().observe(
        &OriginObservation::success(query.clone(), 1_000)
            .with_ttfb_ms(25)
            .with_throughput_bps(9_000_000),
    );

    let loaded = HostStats::from_json(&stats.to_json()).expect("valid model snapshot");
    let estimate = loaded
        .origin_model()
        .estimate(&query, 1_100, DecisionMode::Normal);
    assert!(estimate.effective_samples > 0.0);
    assert!(estimate.throughput_bps.p50 > 0);
}

#[test]
fn loaded_origin_model_reapplies_url_retention_bound() {
    let query = OriginQuery::new(
        "https://persistent.example/video.mp4",
        OriginContext::new(RequestMethod::FullGet, 900_000, MediaClass::WholeObject),
    );
    let mut stats = HostStats::new();
    stats
        .origin_model_mut()
        .observe(&OriginObservation::success(query, 1_000));
    let mut snapshot = serde_json::to_value(stats).expect("serializable stats");
    let urls = snapshot["origin_model"]["urls"]
        .as_array_mut()
        .expect("URL record array");
    let template = urls[0].clone();
    for index in 0..800 {
        let mut record = template.clone();
        record[0]["url_id"] = format!("{index:024x}").into();
        urls.push(record);
    }

    let loaded = HostStats::from_json(&snapshot.to_string()).expect("valid oversized snapshot");
    let normalized = serde_json::to_value(loaded).expect("serializable normalized stats");
    assert_eq!(
        normalized["origin_model"]["urls"]
            .as_array()
            .expect("valid test fixture")
            .len(),
        768
    );
}
