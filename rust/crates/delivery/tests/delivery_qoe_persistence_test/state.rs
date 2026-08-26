#[derive(serde::Deserialize)]
pub(super) struct PersistedQoe {
    pub(super) first_frames: u64,
    pub(super) stall_events: u64,
    pub(super) stall_total_ms: u64,
    pub(super) buffer_samples: u64,
}

pub(super) async fn load_stats(path: &std::path::Path) -> Option<PersistedQoe> {
    let body = tokio::fs::read_to_string(path).await.ok()?;
    let envelope: serde_json::Value = serde_json::from_str(&body).ok()?;
    serde_json::from_value(envelope.get("qoe")?.clone()).ok()
}
