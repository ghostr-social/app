use core::time::Duration;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::origin_model::{
    MediaClass, OriginContext, OriginObservation, OriginQuery, RequestMethod,
};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn seed(root: &Path, primary: &str, alternate: &str, bytes: u64) {
    let at_ms = now_ms();
    let mut stats = HostStats::new();
    let throughput = ThroughputSample::new(12_500_000, Duration::from_secs(1), at_ms, 1)
        .expect("valid test fixture");
    stats.record_overall_throughput(throughput);
    for index in 0..64 {
        let primary_ttfb = if index < 60 { 1 } else { 120 };
        observe(&mut stats, primary, bytes, at_ms, primary_ttfb);
        observe(&mut stats, alternate, bytes, at_ms, 20);
    }
    std::fs::create_dir_all(root).expect("valid test fixture");
    std::fs::write(root.join("host_stats.json"), stats.to_json()).expect("valid test fixture");
}

fn observe(stats: &mut HostStats, source: &str, bytes: u64, at_ms: u64, ttfb_ms: u64) {
    let context = OriginContext::new(RequestMethod::RangeGet, bytes, MediaClass::Unknown)
        .with_observed_at_ms(at_ms);
    let query = OriginQuery::new(source, context);
    let observation = OriginObservation::success(query, at_ms)
        .with_range_compliance(true)
        .with_ttfb_ms(ttfb_ms)
        .with_throughput_bps(100_000_000);
    stats.origin_model_mut().observe(&observation);
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid test fixture")
        .as_millis() as u64
}
