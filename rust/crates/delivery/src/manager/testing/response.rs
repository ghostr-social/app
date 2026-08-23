use crate::manager::DeliveryWorker;
use ghostr_engine::{ActionId, ByteRange, ChunkId, PostId};

impl DeliveryWorker {
    pub(crate) fn register_response_attempt_for_test(
        &mut self,
        post: &PostId,
        source: &str,
    ) -> crate::manager::inflight::ChunkAttempt {
        let identity = self
            .state
            .catalog()
            .transfer_identity(post, source)
            .expect("fixture transfer identity");
        let chunk = ChunkId {
            post: post.clone(),
            range: ByteRange::new(0, 4),
        };
        let attempt = crate::manager::inflight::ChunkAttempt::new(
            chunk,
            identity,
            ActionId::new(self.downloads.next_action_id().value()),
        );
        self.downloads.insert_test_attempt(&attempt);
        attempt
    }

    pub(crate) fn queue_response_for_test(
        &self,
        attempt: crate::manager::inflight::ChunkAttempt,
        response: crate::chunk::downloader::OpenedResponse,
    ) {
        let observed =
            crate::manager::transfers::ObservedResponse::at_network_boundary(attempt, response);
        let event = crate::manager::transfers::TransferEvent::ResponseObserved(Box::new(observed));
        self.ctx
            .events
            .send(crate::manager::transfers::InternalEvent::Transfer(event))
            .unwrap();
    }

    pub(crate) fn queue_cancelled_attempt_for_test(
        &self,
        attempt: crate::manager::inflight::ChunkAttempt,
        source: &str,
    ) {
        attempt.mark_io_finished();
        let result = crate::chunk::downloader::ChunkResult {
            bytes_written: 0,
            range_support: None,
            range_ignored: false,
            cancelled: true,
            total_bytes: None,
            promoted: false,
            request_started: true,
        };
        let event = crate::manager::transfers::chunk_event(attempt, source.into(), Ok(result));
        self.ctx
            .events
            .send(crate::manager::transfers::InternalEvent::Transfer(event))
            .unwrap();
    }

    pub(crate) fn validator_for_test(
        &self,
        post: &PostId,
        source: &str,
    ) -> Option<ghostr_engine::evidence::EvidenceValidator> {
        self.state
            .catalog()
            .lookup(post)?
            .current_validator_for(source)
            .cloned()
    }

    pub(crate) fn catalog_contains_for_test(&self, post: &PostId) -> bool {
        self.state.catalog().lookup(post).is_some()
    }

    pub(crate) async fn finish_probe_result_for_test(
        &mut self,
        post: &PostId,
        source: &str,
        result: crate::probe::media::ProbeResult,
    ) -> Option<ghostr_engine::adaptive::DecisionOutcome> {
        let Some(identity) = self.state.catalog().transfer_identity(post, source) else {
            return None;
        };
        Some(self.finish_probe_result(&identity, result).await)
    }
}
