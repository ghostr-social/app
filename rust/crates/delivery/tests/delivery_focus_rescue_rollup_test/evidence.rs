use core::time::Duration;
use serde::Deserialize;
use std::path::Path;
use tokio::time::Instant;

#[derive(Debug, Deserialize)]
pub(super) struct RescueEvidence {
    pub(super) transport_substitutions: u64,
    pub(super) rank_displacement_total: u64,
    pub(super) rescue_wait_total_ms: u64,
    #[serde(rename = "eta_unavailable_rescues")]
    eta_unavailable: u64,
    #[serde(rename = "eta_too_long_rescues")]
    eta_too_long: u64,
    #[serde(rename = "delivery_failed_rescues")]
    delivery_failed: u64,
    #[serde(rename = "grace_expired_rescues")]
    grace_expired: u64,
}

impl RescueEvidence {
    pub(super) fn reason_counts(&self) -> [u64; 4] {
        [
            self.eta_unavailable,
            self.eta_too_long,
            self.delivery_failed,
            self.grace_expired,
        ]
    }
}

pub(super) async fn wait_for_rollup(root: &Path, count: u64) -> RescueEvidence {
    let path = root.join("qoe_stats.json");
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(evidence) = read(&path).filter(|value| value.transport_substitutions == count) {
            return evidence;
        }
        assert!(
            Instant::now() < deadline,
            "rescue feedback was not persisted"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

fn read(path: &Path) -> Option<RescueEvidence> {
    let bytes = std::fs::read(path).ok()?;
    let envelope = serde_json::from_slice::<Envelope>(&bytes).ok()?;
    Some(envelope.qoe)
}

#[derive(Deserialize)]
struct Envelope {
    qoe: RescueEvidence,
}
