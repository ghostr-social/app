use crate::manager::inflight::ResponseGenerationFence;
use crate::manager::transfers::ChunkDone;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::WholeBodyExhaustion;
use ghostr_engine::representation::HttpGenerationStamp;

impl DeliveryWorker {
    pub(super) fn record_whole_body_limit(&mut self, done: &ChunkDone) {
        let Some(limit) = done.outcome.as_ref().err().and_then(exhaustion) else {
            return;
        };
        let Some(fence) = self.downloads.policy_limit_generation(&done.attempt) else {
            return;
        };
        self.whole_body_limits.record(
            self.state.catalog(),
            done.attempt.identity().clone(),
            limit.maximum_bytes(),
            limit.observed_bytes(),
            generation(fence),
        );
    }
}

fn exhaustion(error: &anyhow::Error) -> Option<WholeBodyExhaustion> {
    if let Some(limit) = crate::chunk::whole_body_limit::from_error(error) {
        return WholeBodyExhaustion::new(limit.maximum_bytes(), limit.received_bytes());
    }
    crate::chunk::whole_body_bound::from_error(error)?.exhaustion()
}

fn generation(fence: ResponseGenerationFence) -> Option<HttpGenerationStamp> {
    match fence {
        ResponseGenerationFence::Durable(lease) => Some(HttpGenerationStamp::from_trusted(lease)),
        ResponseGenerationFence::ActionScoped(generation) => generation,
    }
}
