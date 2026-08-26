use super::QoeStats;
use ghostr_engine::watch_model::WatchModel;
use log::warn;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

const STATE_VERSION: u16 = 1;

#[derive(Default)]
pub struct PlaybackLearningState {
    pub(crate) qoe: QoeStats,
    pub watch: WatchModel,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PersistedLearningState {
    version: u16,
    qoe: QoeStats,
    watch: serde_json::Value,
}

pub async fn load_playback_learning(path: &Path) -> PlaybackLearningState {
    let Ok(json) = tokio::fs::read_to_string(path).await else {
        return PlaybackLearningState::default();
    };
    load_json(&json).unwrap_or_default()
}

/// Atomically persists `QoE` statistics and learned playback behavior.
///
/// # Errors
///
/// Returns an I/O error when the staged state cannot be written or renamed into place.
pub async fn save_playback_learning(
    path: &Path,
    qoe: &QoeStats,
    watch: &WatchModel,
) -> io::Result<()> {
    let staging = path.with_extension("json.tmp");
    let body = encoded_state(qoe, watch);
    if let Err(error) = tokio::fs::write(&staging, body).await {
        remove_staging(&staging).await;
        return Err(error);
    }
    if let Err(error) = tokio::fs::rename(&staging, path).await {
        remove_staging(&staging).await;
        return Err(error);
    }
    Ok(())
}

fn load_json(json: &str) -> Option<PlaybackLearningState> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    if is_envelope(&value) {
        return load_envelope(value);
    }
    Some(PlaybackLearningState {
        qoe: serde_json::from_value(value).ok()?,
        watch: WatchModel::default(),
    })
}

fn is_envelope(value: &serde_json::Value) -> bool {
    value.get("version").is_some() || value.get("qoe").is_some() || value.get("watch").is_some()
}

fn load_envelope(value: serde_json::Value) -> Option<PlaybackLearningState> {
    let persisted: PersistedLearningState = serde_json::from_value(value).ok()?;
    (persisted.version == STATE_VERSION).then_some(())?;
    let watch = WatchModel::from_state_json(&persisted.watch.to_string()).unwrap_or_default();
    Some(PlaybackLearningState {
        qoe: persisted.qoe,
        watch,
    })
}

fn encoded_state(qoe: &QoeStats, watch: &WatchModel) -> Vec<u8> {
    let persisted = PersistedLearningState {
        version: STATE_VERSION,
        qoe: qoe.clone(),
        watch: serde_json::from_str(&watch.state().to_json())
            .expect("watch state always serializes"),
    };
    serde_json::to_vec(&persisted).expect("playback learning state always serializes")
}

async fn remove_staging(path: &Path) {
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => warn!("QoE staging cleanup failed: {error}"),
    }
}
