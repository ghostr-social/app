use crate::delivery_events::{
    ClearRequest, DeliveryCommand, PlaybackPresentation, PlayerPreparationReport,
};
use crate::manager::response_open::ResponseOpenRequest;
use crate::manager::time::unix_time_ms;
use crate::manager::timeline::TimelineResult;
use crate::manager::transfers::{InternalEvent, MaintenanceEvent, TransferEvent};
use crate::manager::DeliveryWorker;
use crate::playback_demand::DemandState;
use tokio::sync::oneshot;

pub(crate) enum Wake {
    Clear(ClearRequest),
    Command(DeliveryCommand),
    Commands(Vec<DeliveryCommand>),
    PlayerPreparation(PlayerPreparationReport),
    PlaybackPresentation(PlaybackPresentation),
    Demand(DemandState),
    Response(Box<ResponseOpenRequest>),
    Internal(InternalEvent),
    Timeline(TimelineResult),
}

type ClearCompletion = (oneshot::Sender<anyhow::Result<()>>, anyhow::Result<()>);

impl DeliveryWorker {
    pub(crate) async fn step(&mut self) -> bool {
        let Some(wake) = self.next_wake().await else {
            return false;
        };
        let clear = self.apply(wake).await;
        if clear.is_none() {
            self.apply_pending_focus().await;
        }
        self.reconcile().await;
        complete_clear(clear);
        true
    }

    async fn apply_pending_focus(&mut self) {
        let Some(commands) = self.commands.try_controls_through_focus() else {
            return;
        };
        self.wake_cursor
            .observe(crate::manager::wake_lane::WakeLane::Control);
        self.apply_commands(commands).await;
    }

    async fn apply(&mut self, wake: Wake) -> Option<ClearCompletion> {
        match wake {
            Wake::Clear(reply) => Some((reply, self.clear().await)),
            Wake::Command(command) => {
                self.apply_command(command).await;
                None
            }
            Wake::Commands(commands) => {
                self.apply_commands(commands).await;
                None
            }
            Wake::PlayerPreparation(report) => {
                self.apply_player_preparation_feedback(report);
                None
            }
            Wake::PlaybackPresentation(event) => {
                self.apply_presentation(event);
                None
            }
            Wake::Demand(signal) => {
                self.demand_leases.apply(signal);
                None
            }
            Wake::Response(response) => {
                self.apply_response_open(*response).await;
                None
            }
            Wake::Internal(event) => {
                self.apply_internal(event).await;
                None
            }
            Wake::Timeline(result) => {
                self.timelines.stage(result);
                None
            }
        }
    }

    async fn apply_commands(&mut self, commands: Vec<DeliveryCommand>) {
        for command in commands {
            self.apply_command(command).await;
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
            DeliveryCommand::NetworkChanged => self.note_network_profile_change(),
            DeliveryCommand::StorageChanged => {}
        }
        self.prune_scheduling_history();
        self.bind_representations().await;
        self.qoe.schedule_save(&self.ctx.events);
    }

    fn apply_focus_command(&mut self, focus: crate::delivery_events::DeliveryFocus) {
        let previous = self.state.focus().current().cloned();
        let segmented_focus = focus.clone();
        let observed_at_ms = unix_time_ms();
        if !self.state.apply_focus(focus, observed_at_ms) {
            return;
        }
        self.qoe.note_focus(&segmented_focus, observed_at_ms);
        self.segmented.set_startup_eta_ms(self.qoe.startup_eta_ms());
        self.segmented.apply_focus(&segmented_focus);
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
        self.timelines.retain_history(&retained);
    }

    pub(crate) async fn bind_representations(&mut self) {
        for binding in self.state.take_representation_bindings() {
            self.downloads.cancel_obsolete(&binding);
            if let Err(error) = self.ctx.store.bind_representation(binding).await {
                log::warn!("Video representation binding failed: {error:#}");
            }
        }
    }

    async fn apply_internal(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::ImmediateReplan => self.consume_immediate_replan(),
            InternalEvent::Transfer(transfer) => self.apply_transfer(transfer).await,
            InternalEvent::Segmented(done) => self.segmented.finish(done),
            InternalEvent::Maintenance(maintenance) => self.apply_maintenance(maintenance).await,
            InternalEvent::TrafficChanged => self.absorb_traffic(),
        }
    }

    async fn apply_transfer(&mut self, event: TransferEvent) {
        match event {
            TransferEvent::ChunkDone(done) => self.finish_chunk(done).await,
            TransferEvent::ProbeDone(done) => self.finish_probe(done).await,
            TransferEvent::ResponseObserved(observed) => self.observe_response(observed),
        }
    }

    async fn apply_maintenance(&mut self, event: MaintenanceEvent) {
        match event {
            MaintenanceEvent::CooldownOver(post, cooldown) => {
                self.cooldown_timers.finish(&post, cooldown);
                self.retry.warm_up(&post, cooldown);
            }
            MaintenanceEvent::SaveStats => {
                self.keeper.save_now().await;
                let evidence = self.state.catalog().evidence_state();
                self.reliability.save_now(&evidence).await;
                self.save_capability().await;
            }
            MaintenanceEvent::SaveQoe => self.qoe.save_now().await,
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
