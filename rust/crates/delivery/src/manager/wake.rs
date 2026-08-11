use crate::delivery_events::{ClearRequest, DeliveryCommand};
use crate::manager::transfers::InternalEvent;
use crate::manager::DeliveryWorker;
use crate::playback_demand::DemandSignal;
use ghostr_engine::concurrency::NetworkSetback;
use ghostr_engine::playback::PlaybackPhase;
use tokio::sync::oneshot;

pub(super) enum Wake {
    Clear(ClearRequest),
    Command(DeliveryCommand),
    Demand(DemandSignal),
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
                self.pending_demand = Some(signal);
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
            DeliveryCommand::Prioritize(post) => self.state.prioritize(post),
            DeliveryCommand::Focus(focus) => {
                let _ = self.state.apply_focus(focus);
            }
            DeliveryCommand::Playback(playback) => self.apply_playback(playback),
            DeliveryCommand::Config(level) => {
                self.state.apply_level(level);
                self.update_concurrency_ceiling();
            }
        }
        self.prune_scheduling_history();
        self.bind_representations().await;
    }

    fn prune_scheduling_history(&mut self) {
        let retained = self.state.retained_posts();
        self.probes.retain_history(&retained);
        self.retry.retain_history(&retained);
    }

    async fn bind_representations(&mut self) {
        for binding in self.state.take_representation_bindings() {
            if let Err(error) = self.ctx.store.bind_representation(binding).await {
                log::warn!("Video representation binding failed: {error:#}");
            }
        }
    }

    fn apply_playback(&mut self, playback: crate::delivery_events::DeliveryPlayback) {
        let stalled = playback.observation.phase() == PlaybackPhase::NetworkStalled;
        if self.state.apply_playback(playback) && stalled {
            self.note_network_setback(NetworkSetback::Stall);
        }
    }

    async fn apply_internal(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::ChunkDone(done) => self.finish_chunk(done).await,
            InternalEvent::ProbeDone(done) => self.finish_probe(done).await,
            InternalEvent::CooldownOver(post) => self.retry.warm_up(&post),
            InternalEvent::SaveStats => self.keeper.save_now().await,
            InternalEvent::StoreCapacityChanged(generation) => {
                self.resume_store_capacity(generation);
            }
            InternalEvent::TrafficChanged => self.absorb_traffic(),
        }
    }
}

fn complete_clear(clear: Option<ClearCompletion>) {
    if let Some((reply, result)) = clear {
        let _ = reply.send(result);
    }
}
