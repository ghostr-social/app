use crate::manager::origin_admission::cap_request;
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};

#[test]
fn uncertain_whole_fetch_is_reduced_to_one_sparse_prefix_probe() {
    let request = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped {
            maximum_bytes: 8 * 1024 * 1024,
        },
        reason: WholeFetchReason::PlannedCompletion,
    };

    assert_eq!(
        cap_request(request, 65_536),
        RetrievalRequest::FetchRange {
            bytes: ghostr_engine::ByteRange::new(0, 65_536),
            promotion: None,
        }
    );
}
