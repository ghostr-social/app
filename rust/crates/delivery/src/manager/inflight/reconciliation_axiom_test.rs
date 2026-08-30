use super::*;

impl InFlightChunks {
    /// Retains planned IO, then reserves slots for higher-priority work.
    pub(crate) fn reconcile(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        self.reconcile_with_commitments(planned, capacity, &HashSet::new());
    }
}

#[test]
fn promotion_metadata_does_not_restart_the_same_range() {
    let bytes = ghostr_engine::ByteRange::new(0, 65_536);
    let normalized = plain_range(bytes);
    let first = promoted_range(bytes, 1_000);
    let revised = promoted_range(bytes, 2_000);

    assert!(retrieval_matches(normalized, first));
    assert!(retrieval_matches(first, revised));
}

#[test]
fn response_promotion_does_not_restart_the_wire_range() {
    let bytes = ghostr_engine::ByteRange::new(0, 65_536);

    assert!(retrieval_action_matches(
        plain_range(bytes),
        promoted_whole(),
        plain_range(bytes),
    ));
}

fn plain_range(bytes: ghostr_engine::ByteRange) -> ghostr_engine::adaptive::RetrievalRequest {
    ghostr_engine::adaptive::RetrievalRequest::FetchRange {
        bytes,
        promotion: None,
    }
}

fn promoted_range(
    bytes: ghostr_engine::ByteRange,
    valid_until_ms: u64,
) -> ghostr_engine::adaptive::RetrievalRequest {
    ghostr_engine::adaptive::RetrievalRequest::FetchRange {
        bytes,
        promotion: Some(ghostr_engine::adaptive::PromotionGrant {
            maximum_bytes: 293_999,
            valid_until_ms,
        }),
    }
}

fn promoted_whole() -> ghostr_engine::adaptive::RetrievalRequest {
    ghostr_engine::adaptive::RetrievalRequest::FetchWhole {
        contract: ghostr_engine::adaptive::WholeBodyContract::Capped {
            maximum_bytes: 293_999,
        },
        reason: ghostr_engine::adaptive::WholeFetchReason::PromotedResponse,
    }
}
