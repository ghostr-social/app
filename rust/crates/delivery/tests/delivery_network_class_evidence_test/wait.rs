use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use std::path::Path;

pub async fn for_network_evidence(path: &Path, url: &str) -> HostStats {
    loop {
        if let Some(stats) = load(path) {
            if samples(&stats, url, NetworkClass::Wifi) > 0.0
                && samples(&stats, url, NetworkClass::Cellular) > 0.0
            {
                return stats;
            }
        }
        tokio::task::yield_now().await;
    }
}

fn load(path: &Path) -> Option<HostStats> {
    let json = std::fs::read_to_string(path).ok()?;
    HostStats::from_json(&json).ok()
}

fn samples(stats: &HostStats, url: &str, network: NetworkClass) -> f64 {
    let now = now_ms();
    let query = OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::FullGet, 16, MediaClass::Unknown)
            .with_network(network)
            .with_observed_at_ms(now),
    );
    stats
        .origin_model()
        .estimate(&query, now, DecisionMode::Normal)
        .effective_samples
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("valid test fixture")
        .as_millis() as u64
}
