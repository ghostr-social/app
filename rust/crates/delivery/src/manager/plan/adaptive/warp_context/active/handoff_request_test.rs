use super::exact_provisional_bootstrap;
use ghostr_engine::adaptive::{
    RetrievalRequest, WholeBodyContract, WholeFetchReason, MEDIA_BOOTSTRAP_PROBE_BYTES,
};
use ghostr_engine::ByteRange;

#[test]
fn only_the_exact_bounded_prefix_is_a_handoff_block() {
    assert!(exact_provisional_bootstrap(range(ByteRange::new(
        0,
        MEDIA_BOOTSTRAP_PROBE_BYTES,
    ))));
    assert!(!exact_provisional_bootstrap(range(ByteRange::new(
        1,
        MEDIA_BOOTSTRAP_PROBE_BYTES,
    ))));
    assert!(!exact_provisional_bootstrap(range(ByteRange::new(
        0,
        MEDIA_BOOTSTRAP_PROBE_BYTES + 1,
    ))));
    assert!(!exact_provisional_bootstrap(RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped {
            maximum_bytes: MEDIA_BOOTSTRAP_PROBE_BYTES,
        },
        reason: WholeFetchReason::PromotedResponse,
    }));
}

fn range(bytes: ByteRange) -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes,
        promotion: None,
    }
}
