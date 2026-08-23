use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_OBSERVATION_ORDER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

pub(crate) fn evidence_time() -> ghostr_engine::evidence::EvidenceTime {
    let order = NEXT_OBSERVATION_ORDER.fetch_add(1, Ordering::Relaxed);
    ghostr_engine::evidence::EvidenceTime::ordered(unix_time_ms(), order)
}
