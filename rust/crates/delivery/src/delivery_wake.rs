use crate::delivery_events::{ClearRequest, DeliveryCommand};
use crate::delivery_manager::DeliveryWorker;
use crate::delivery_transfers::InternalEvent;
use crate::playback_demand::DemandSignal;
use tokio::sync::oneshot;

enum Wake {
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

    async fn next_wake(&mut self) -> Option<Wake> {
        let (commands, clears) = self.commands.receivers();
        tokio::select! {
            biased;
            Some(clear) = clears.recv() => Some(Wake::Clear(clear)),
            command = commands.recv() => command.map(Wake::Command),
            Some(signal) = self.demand.recv() => Some(Wake::Demand(signal)),
            Some(event) = self.events.recv() => Some(Wake::Internal(event)),
        }
    }

    async fn apply(&mut self, wake: Wake) -> Option<ClearCompletion> {
        match wake {
            Wake::Clear(reply) => Some((reply, self.clear().await)),
            Wake::Command(command) => {
                self.apply_command(command);
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

    fn apply_command(&mut self, command: DeliveryCommand) {
        match command {
            DeliveryCommand::Candidate(candidate) => self.state.apply_candidate(candidate),
            DeliveryCommand::Prioritize(post) => self.state.prioritize(post),
            DeliveryCommand::Focus(focus) => self.state.apply_focus(focus),
            DeliveryCommand::Config(level) => self.state.apply_level(level),
        }
    }

    async fn apply_internal(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::ChunkDone(done) => self.finish_chunk(done).await,
            InternalEvent::ProbeDone(done) => self.finish_probe(done).await,
            InternalEvent::CooldownOver(post) => self.retry.warm_up(&post),
            InternalEvent::SaveStats => self.keeper.save_now().await,
        }
    }
}

fn complete_clear(clear: Option<ClearCompletion>) {
    if let Some((reply, result)) = clear {
        let _ = reply.send(result);
    }
}
