use crate::chunk::downloader::ChunkSpec;
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::TransferTimeouts;

pub(super) fn spec(requests: &MediaRequestExecutor) -> ChunkSpec<'_> {
    ChunkSpec {
        requests,
        url: "https://media.example/video.mp4",
        request: RetrievalRequest::FetchRange {
            bytes: ByteRange::new(0, 1),
            promotion: None,
        },
        attempt_profile: crate::tests::support::range_profile(1),
        priority: PreemptionAuthority::Transition,
        continuation: None,
        timeouts: TransferTimeouts::default(),
    }
}
