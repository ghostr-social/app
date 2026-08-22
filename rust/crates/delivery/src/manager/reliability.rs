use crate::manager::transfers::{InternalEvent, MaintenanceEvent};
use ghostr_engine::catalog::CatalogEvidenceState;
use ghostr_engine::evidence::FieldReliabilityModel;
use log::warn;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct ReliabilityKeeper {
    path: PathBuf,
    debounce: Duration,
    seen_revision: u64,
    dirty: bool,
    save_pending: bool,
}

impl ReliabilityKeeper {
    pub(crate) async fn load(path: PathBuf, debounce: Duration) -> (Self, CatalogEvidenceState) {
        let state = load_catalog_evidence(&path).await;
        let keeper = Self {
            path,
            debounce,
            seen_revision: 0,
            dirty: false,
            save_pending: false,
        };
        (keeper, state)
    }

    pub(crate) fn observe(&mut self, revision: u64, events: &UnboundedSender<InternalEvent>) {
        if revision == self.seen_revision {
            return;
        }
        self.seen_revision = revision;
        self.dirty = true;
        self.schedule_save(events);
    }

    pub(crate) async fn save_now(&mut self, state: &CatalogEvidenceState) {
        self.save_pending = false;
        if !self.dirty {
            return;
        }
        match save_catalog_evidence(&self.path, state).await {
            Ok(()) => self.dirty = false,
            Err(error) => warn!("Field-reliability snapshot failed: {error}"),
        }
    }

    fn schedule_save(&mut self, events: &UnboundedSender<InternalEvent>) {
        if self.save_pending {
            return;
        }
        self.save_pending = true;
        let events = events.clone();
        let debounce = self.debounce;
        tokio::spawn(async move {
            tokio::time::sleep(debounce).await;
            let _ = events.send(InternalEvent::Maintenance(MaintenanceEvent::SaveStats));
        });
    }
}

pub(crate) async fn load_catalog_evidence(path: &Path) -> CatalogEvidenceState {
    let Ok(json) = tokio::fs::read_to_string(path).await else {
        return CatalogEvidenceState::default();
    };
    CatalogEvidenceState::from_json(&json).unwrap_or_else(|_| {
        FieldReliabilityModel::from_json(&json)
            .map(CatalogEvidenceState::from_reliability)
            .unwrap_or_default()
    })
}

pub(crate) async fn save_catalog_evidence(
    path: &Path,
    state: &CatalogEvidenceState,
) -> io::Result<()> {
    save_json(path, state.to_json()).await
}

#[cfg(test)]
pub(crate) async fn load_field_reliability(path: &Path) -> FieldReliabilityModel {
    match tokio::fs::read_to_string(path).await {
        Ok(json) => FieldReliabilityModel::from_json(&json).unwrap_or_default(),
        Err(_) => FieldReliabilityModel::default(),
    }
}

#[cfg(test)]
pub(crate) async fn save_field_reliability(
    path: &Path,
    model: &FieldReliabilityModel,
) -> io::Result<()> {
    save_json(path, model.to_json()).await
}

async fn save_json(path: &Path, json: String) -> io::Result<()> {
    let staging = path.with_extension("json.tmp");
    tokio::fs::write(&staging, json).await?;
    if let Err(error) = tokio::fs::rename(&staging, path).await {
        let _ = tokio::fs::remove_file(&staging).await;
        return Err(error);
    }
    Ok(())
}
