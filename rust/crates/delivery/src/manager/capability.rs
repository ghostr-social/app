use crate::client_capability::{
    load_client_capabilities, save_client_capabilities, ClientCapabilityModel,
};
use crate::delivery_events::{PlayerPreparationActorOutcome, PlayerPreparationEnvelope};
use crate::manager::time::unix_time_ms;
use crate::manager::transfers::{InternalEvent, MaintenanceEvent};
use crate::manager::DeliveryWorker;
use log::warn;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub(super) struct CapabilityKeeper {
    path: PathBuf,
    debounce: Duration,
    seen_revision: u64,
    dirty: bool,
    save_pending: bool,
}

impl CapabilityKeeper {
    pub(super) async fn load(path: PathBuf, debounce: Duration) -> (Self, ClientCapabilityModel) {
        let model = load_client_capabilities(&path).await;
        let keeper = Self {
            path,
            debounce,
            seen_revision: model.revision(),
            dirty: false,
            save_pending: false,
        };
        (keeper, model)
    }

    pub(super) fn observe(&mut self, revision: u64, events: &UnboundedSender<InternalEvent>) {
        if revision == self.seen_revision {
            return;
        }
        self.seen_revision = revision;
        self.dirty = true;
        self.schedule_save(events);
    }

    pub(super) async fn save_now(&mut self, model: &ClientCapabilityModel) {
        self.save_pending = false;
        if !self.dirty {
            return;
        }
        match save_client_capabilities(&self.path, model).await {
            Ok(()) => self.dirty = false,
            Err(error) => warn!("Client-capability snapshot failed: {error}"),
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

impl DeliveryWorker {
    pub(super) fn apply_player_preparation_feedback(
        &mut self,
        envelope: PlayerPreparationEnvelope,
    ) {
        let outcome = self
            .state
            .apply_player_preparation_at(envelope.report().clone(), unix_time_ms());
        if outcome == PlayerPreparationActorOutcome::Applied {
            self.capability
                .observe(self.state.client_capability_revision(), &self.ctx.events);
        }
        self.commands.complete_player_preparation(envelope, outcome);
    }

    pub(super) async fn save_capability(&mut self) {
        self.capability
            .save_now(self.state.client_capabilities())
            .await;
    }
}
