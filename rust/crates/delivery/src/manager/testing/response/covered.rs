use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{ActionRegistration, ChunkAttempt};
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::{PreemptionAuthority, PromotionGrant, RetrievalRequest};
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use ghostr_partial_store::partial_range_store::StoreAction;

impl DeliveryWorker {
    pub(crate) async fn register_covered_response_for_test(
        &mut self,
        post: &PostId,
        source: &str,
        valid_until_ms: u64,
    ) -> (ChunkAttempt, StoreAction) {
        let identity = self
            .state
            .catalog()
            .transfer_identity(post, source)
            .expect("fixture transfer identity");
        let chunk = ChunkId {
            post: post.clone(),
            range: ByteRange::new(0, 8),
        };
        let action = self.downloads.next_action_id();
        let attempt = ChunkAttempt::new(chunk.clone(), identity.clone(), action);
        let store_action = self
            .ctx
            .store
            .reserve_action(&identity, action.value(), 8)
            .await
            .expect("fixture storage reservation");
        let (handle, _token) = cancel_pair();
        self.downloads.insert_test_action(ActionRegistration {
            attempt: &attempt,
            priority: priority(chunk),
            retrieval: ranged(valid_until_ms),
            host: "unused.example".into(),
            committed_until_ms: valid_until_ms,
            launched_at_ms: valid_until_ms.saturating_sub(1),
            handle,
            store_action: Some(store_action.clone()),
            committed_network_bytes: Some(8),
            admission_claim: None,
        });
        (attempt, store_action)
    }
}

fn priority(chunk: ChunkId) -> RangeRequest {
    RangeRequest {
        chunk,
        authority: PreemptionAuthority::Transition,
        score: 1.0,
        contiguous_depth_bytes: 0,
    }
}

fn ranged(valid_until_ms: u64) -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 8),
        promotion: Some(PromotionGrant {
            maximum_bytes: 16,
            valid_until_ms,
        }),
    }
}
