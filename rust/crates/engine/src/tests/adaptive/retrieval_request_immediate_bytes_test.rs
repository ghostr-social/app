use crate::adaptive::{PromotionGrant, RetrievalRequest, WholeBodyContract, WholeFetchReason};
use crate::ByteRange;

#[test]
fn latent_promotion_is_not_part_of_the_initial_resource_commit() {
    let range = RetrievalRequest::FetchRange {
        bytes: ByteRange::new(4, 8),
        promotion: Some(PromotionGrant {
            maximum_bytes: 16,
            valid_until_ms: 99,
        }),
    };
    let whole = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped { maximum_bytes: 16 },
        reason: WholeFetchReason::PlannedCompletion,
    };

    assert_eq!(range.immediate_network_bytes(), 4);
    assert_eq!(whole.immediate_network_bytes(), 16);
    assert_eq!(range.reserved_network_bytes(), 16);
}
