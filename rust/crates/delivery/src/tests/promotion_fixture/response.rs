use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseWriteMode,
};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};

pub(crate) fn response() -> OpenedResponse {
    let contract = WholeBodyContract::Exact { expected_bytes: 16 };
    OpenedResponse::new(
        ResponseObservation::Body {
            request: RetrievalRequest::FetchWhole {
                contract,
                reason: WholeFetchReason::PromotedResponse,
            },
            total: Some(16),
            range_support: Some(false),
            promoted: true,
        },
        None,
        ResponseWriteMode::SingleResponse(contract),
        HttpResponseEvidence {
            final_url: "https://origin.test/video".into(),
            status: 200,
            content_type: Some("video/mp4".into()),
            validator: None,
            observed: 0.into(),
        },
    )
}
