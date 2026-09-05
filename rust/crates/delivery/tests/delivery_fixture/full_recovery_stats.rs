use core::time::Duration;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    Admission, ErrorReason, MediaClass, OriginContext, OriginObservation, OriginQuery,
    RequestMethod,
};
use std::path::Path;

const EVIDENCE_TIMEOUT: Duration = Duration::from_secs(30);

pub fn seed(root: &Path, urls: &[(&str, u64)]) {
    let now = unix_time_ms();
    let mut stats = HostStats::new();
    for (url, bytes) in urls {
        let query = query(url, *bytes, now - 6_000);
        for _ in 0..4_096 {
            stats.origin_model_mut().observe(
                &OriginObservation::success(query.clone(), now - 6_000)
                    .with_ttfb_ms(1)
                    .with_throughput_bps(100_000_000),
            );
        }
    }
    let failure = query(urls[0].0, urls[0].1, now - 5_000);
    for offset in 0..3 {
        stats
            .origin_model_mut()
            .observe(&OriginObservation::failure(
                failure.clone(),
                now - 5_000 + offset,
                ErrorReason::Timeout,
            ));
    }
    std::fs::create_dir_all(root).expect("create stats root");
    std::fs::write(root.join("host_stats.json"), stats.to_json()).expect("write stats");
}

pub fn query(url: &str, bytes: u64, observed_at_ms: u64) -> OriginQuery {
    OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::FullGet, bytes, MediaClass::Unknown)
            .with_observed_at_ms(observed_at_ms),
    )
}

pub fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_millis() as u64
}

pub async fn assert_admission(path: &Path, query: &OriginQuery, expected: Admission) {
    let json = tokio::fs::read_to_string(path)
        .await
        .expect("persisted host stats");
    let stats = HostStats::from_json(&json).expect("valid host stats");
    assert_eq!(
        stats
            .origin_model()
            .circuit_admission(query, unix_time_ms()),
        expected,
        "completed trial updates delivery statistics"
    );
}

pub async fn wait_for_admission(path: &Path, query: &OriginQuery, expected: Admission) {
    let label = match expected {
        Admission::RecoveryTrial => "recovery probe completion evidence",
        Admission::Production => "recovery trial completion evidence",
        _ => "recovery circuit evidence",
    };
    crate::delivery_fixture::stats::wait_for_within(path, EVIDENCE_TIMEOUT, label, |stats| {
        stats
            .origin_model()
            .circuit_admission(query, unix_time_ms())
            == expected
    })
    .await;
}
