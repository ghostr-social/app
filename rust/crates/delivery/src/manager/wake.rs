use crate::delivery_events::{
    ClearRequest, DeliveryCommand, PlaybackPresentation, PlayerPreparationEnvelope,
};
use crate::manager::response_open::ResponseOpenRequest;
use crate::manager::time::unix_time_ms;
use crate::manager::timeline::TimelineResult;
use crate::manager::transfers::InternalEvent;
use crate::manager::DeliveryWorker;
use crate::playback_demand::DemandState;

mod clear;
mod internal;
mod network;
mod response;
use clear::ClearCompletion;

pub(crate) enum Wake {
    Clear(ClearRequest),
    Command(DeliveryCommand),
    Commands(Vec<DeliveryCommand>),
    PlayerPreparation(PlayerPreparationEnvelope),
    PlaybackPresentation(PlaybackPresentation),
    Demand(DemandState),
    Response(Box<ResponseOpenRequest>),
    Internal(InternalEvent),
    ControlInterval,
    SegmentedInvalidated,
    Timeline(TimelineResult),
}
impl DeliveryWorker {
    pub(crate) async fn step(&mut self) -> bool {
        let Some(wake) = self.next_wake().await else {
            return false;
        };
        let wake = match wake {
            Wake::Response(response) => {
                self.step_response(*response).await;
                return true;
            }
            wake => wake,
        };
        let clear = self.apply(wake).await;
        if clear.is_none() {
            self.apply_pending_focus().await;
        }
        Box::pin(self.reconcile()).await;
        clear::complete(clear);
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
            Wake::PlayerPreparation(envelope) => {
                self.apply_player_preparation_feedback(envelope).await;
                None
            }
            Wake::PlaybackPresentation(event) => {
                self.apply_presentation(&event);
                None
            }
            Wake::Demand(signal) => {
                self.demand_leases.apply(signal);
                None
            }
            Wake::Response(_) => unreachable!("response wakes have a staged reconciliation"),
            Wake::Internal(event) => {
                self.apply_internal(event).await;
                None
            }
            Wake::ControlInterval => None,
            Wake::SegmentedInvalidated => {
                self.segmented.reseed_invalidated();
                self.reconcile_segmented_roots();
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
        self.refresh_observation_posts();
        match command {
            DeliveryCommand::Candidate(candidate) => self.state.apply_candidate(candidate),
            DeliveryCommand::Focus(focus) => self.apply_focus_command(focus),
            DeliveryCommand::Playback(playback) => self.apply_playback(&playback),
            DeliveryCommand::Config(level) => {
                self.state.apply_level(level);
                self.update_concurrency_ceiling();
            }
            DeliveryCommand::NetworkStatus(status) => self.apply_network_status(status),
            DeliveryCommand::NetworkProfile {
                generation,
                profile,
            } => self.apply_network_profile(generation, profile),
            DeliveryCommand::StorageChanged => {}
        }
        self.prune_scheduling_history();
        self.bind_representations().await;
        self.qoe.schedule_save(&self.ctx.events);
    }

    fn apply_focus_command(&mut self, focus: crate::delivery_events::DeliveryFocus) {
        let previous = self.state.focus().current().cloned();
        let segmented_focus = focus.clone();
        let hls_changed = self.segmented.changed_hls_sources(&segmented_focus);
        let hls_cooldown_resets = self.segmented.hls_cooldown_resets(&segmented_focus);
        let observed_at_ms = unix_time_ms();
        if !self.state.apply_focus(focus, observed_at_ms) {
            return;
        }
        self.qoe.note_focus(&segmented_focus, observed_at_ms);
        self.segmented.set_startup_eta_ms(self.qoe.startup_eta_ms());
        let segmented_changed = self.segmented.apply_focus(&segmented_focus);
        let current = self.state.focus().current().cloned();
        let progressive_changed = self.state.take_changed_representations();
        let hls_restarts = self.reset_focus_representations(
            progressive_changed,
            hls_changed,
            &hls_cooldown_resets,
        );
        self.apply_retry_focus_change(previous.as_ref(), current.as_ref());
        if segmented_changed {
            self.restart_segmented_roots(&hls_restarts);
            self.reconcile_segmented_roots();
        }
        self.focus_lease
            .pin(self.ctx.store.as_ref(), current.as_ref());
        self.pressure.focus_changed();
    }

    fn prune_scheduling_history(&mut self) {
        let retained = self.state.retained_posts();
        self.retain_transform_jobs(&retained);
        self.probes.retain_history(&retained);
        self.retry.retain_history(&retained);
        self.cooldown_timers.retain(&retained);
        self.timelines.retain_history(&retained);
    }

    pub(super) async fn bind_representations(&mut self) {
        for binding in self.state.take_representation_bindings() {
            self.cancel_obsolete_transform(&binding);
            self.downloads.cancel_obsolete(&binding);
            if let Err(error) = self.ctx.store.bind_representation(binding).await {
                log::warn!("Video representation binding failed: {error:#}");
            }
        }
    }
}
