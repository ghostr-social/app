use crate::delivery_events::{ClearRequest, DeliveryCommand};
use crate::manager::concurrency::network_profile_setback;
use crate::manager::time::unix_time_ms;
use crate::manager::transfers::{InternalEvent, MaintenanceEvent, TransferEvent};
use crate::manager::DeliveryWorker;
use crate::playback_demand::DemandState;
use ghostr_engine::concurrency::NetworkSetback;
use ghostr_engine::playback::PlaybackPhase;
use tokio::sync::oneshot;

pub(crate) enum Wake {
    Clear(ClearRequest),
    Command(DeliveryCommand),
    Demand(DemandState),
    Internal(InternalEvent),
}

type ClearCompletion = (oneshot::Sender<anyhow::Result<()>>, anyhow::Result<()>);

impl DeliveryWorker {
    pub(crate) async fn step(&mut self) -> bool {
        let Some(wake) = self.next_wake().await else {
            return false;
        };
        let clear = self.apply(wake).await;
        self.reconcile().await;
        complete_clear(clear);
        true
    }

    async fn apply(&mut self, wake: Wake) -> Option<ClearCompletion> {
        match wake {
            Wake::Clear(reply) => Some((reply, self.clear().await)),
            Wake::Command(command) => {
                self.apply_command(command).await;
                None
            }
            Wake::Demand(signal) => {
                self.demand_leases.apply(signal);
                None
            }
            Wake::Internal(event) => {
                self.apply_internal(event).await;
                None
            }
        }
    }

    async fn apply_command(&mut self, command: DeliveryCommand) {
        match command {
            DeliveryCommand::Candidate(candidate) => self.state.apply_candidate(candidate),
            DeliveryCommand::Focus(focus) => self.apply_focus_command(focus),
            DeliveryCommand::Playback(playback) => self.apply_playback(playback),
            DeliveryCommand::Config(level) => {
                self.state.apply_level(level);
                self.update_concurrency_ceiling();
            }
            DeliveryCommand::NetworkChanged => {
                let loss = self.ctx.network.profile().packet_loss_bps;
                self.note_network_setback(network_profile_setback(loss));
            }
            DeliveryCommand::StorageChanged => {}
        }
        self.prune_scheduling_history();
        self.bind_representations().await;
    }

    fn apply_focus_command(&mut self, focus: crate::delivery_events::DeliveryFocus) {
        let previous = self.state.focus().current().cloned();
        if !self.state.apply_focus(focus, unix_time_ms()) {
            return;
        }
        let current = self.state.focus().current().cloned();
        for post in self.state.take_changed_representations() {
            self.cooldown_timers.cancel(&post);
            self.probes.representation_changed(&post);
            self.retry.representation_changed(&post);
        }
        self.retry
            .focus_changed(previous.as_ref(), current.as_ref());
        if previous != current {
            if let Some(current) = current.as_ref() {
                self.cooldown_timers.cancel(current);
            }
        }
        self.focus_lease
            .pin(self.ctx.store.as_ref(), current.as_ref());
        self.pressure.focus_changed();
    }

    fn prune_scheduling_history(&mut self) {
        let retained = self.state.retained_posts();
        self.probes.retain_history(&retained);
        self.retry.retain_history(&retained);
        self.cooldown_timers.retain(&retained);
    }

    pub(crate) async fn bind_representations(&mut self) {
        for binding in self.state.take_representation_bindings() {
            self.downloads.cancel_obsolete(&binding);
            if let Err(error) = self.ctx.store.bind_representation(binding).await {
                log::warn!("Video representation binding failed: {error:#}");
            }
        }
    }

    fn apply_playback(&mut self, playback: crate::delivery_events::DeliveryPlayback) {
        let stalled = playback.observation.phase() == PlaybackPhase::NetworkStalled;
        let post = playback.session.post().clone();
        let admission = self.state.apply_playback(playback);
        self.commands.record_playback_admission(admission, &post);
        if admission.is_accepted() && stalled {
            self.note_network_setback(NetworkSetback::Stall);
        }
    }

    async fn apply_internal(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::Transfer(transfer) => self.apply_transfer(transfer).await,
            InternalEvent::Maintenance(maintenance) => self.apply_maintenance(maintenance).await,
            InternalEvent::TrafficChanged => self.absorb_traffic(),
        }
    }

    async fn apply_transfer(&mut self, event: TransferEvent) {
        match event {
            TransferEvent::ChunkDone(done) => self.finish_chunk(done).await,
            TransferEvent::BodyFinished(identity) => self.finish_body(&identity),
            TransferEvent::ProbeDone(done) => self.finish_probe(done).await,
        }
    }

    async fn apply_maintenance(&mut self, event: MaintenanceEvent) {
        match event {
            MaintenanceEvent::CooldownOver(post, cooldown) => {
                self.cooldown_timers.finish(&post, cooldown);
                self.retry.warm_up(&post, cooldown);
            }
            MaintenanceEvent::SaveStats => self.keeper.save_now().await,
            MaintenanceEvent::StoreCapacityChanged(generation) => {
                self.resume_store_capacity(generation);
            }
        }
    }
}

fn complete_clear(clear: Option<ClearCompletion>) {
    if let Some((reply, result)) = clear {
        let _ = reply.send(result);
    }
}
