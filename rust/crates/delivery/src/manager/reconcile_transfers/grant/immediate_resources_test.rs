use super::request_resources;
use ghostr_engine::adaptive::{PromotionGrant, ResourceCost, RetrievalRequest};
use ghostr_engine::ByteRange;

#[test]
fn latent_promotion_commits_only_the_initial_range_at_launch() {
    let request = RetrievalRequest::FetchRange {
        bytes: ByteRange::new(4, 8),
        promotion: Some(PromotionGrant {
            maximum_bytes: 16,
            valid_until_ms: 100,
        }),
    };

    assert_eq!(request_resources(request), ResourceCost::new(4, 4, 0, 1));
}
