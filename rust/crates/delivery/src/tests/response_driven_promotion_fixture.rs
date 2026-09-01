use crate::chunk::downloader::{
    HttpResponseEvidence, OpenedResponse, ResponseObservation, ResponseWriteMode,
};
use crate::delivery_events::DeliveryHandle;
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::{DeliveryKind, VideoMeta};

pub(super) const SOURCE: &str = "https://unused.example/video.mp4";

pub(super) fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}

pub(super) fn response() -> OpenedResponse {
    let contract = WholeBodyContract::Exact { expected_bytes: 8 };
    OpenedResponse::new(
        ResponseObservation::Body {
            request: RetrievalRequest::FetchWhole {
                contract,
                reason: WholeFetchReason::PromotedResponse,
            },
            total: Some(8),
            range_support: Some(false),
            promoted: true,
        },
        None,
        ResponseWriteMode::SingleResponse(contract),
        HttpResponseEvidence {
            final_url: SOURCE.into(),
            status: 200,
            content_type: Some("video/mp4".into()),
            validator: None,
            observed: 0.into(),
        },
    )
}

pub(super) fn promoted_request() -> RetrievalRequest {
    RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact { expected_bytes: 8 },
        reason: WholeFetchReason::PromotedResponse,
    }
}

pub(super) fn assert_promotion_trace(handle: &DeliveryHandle, action_id: u64, valid_until_ms: u64) {
    let json = handle.decision_history_json().expect("decision JSON");
    let evidence: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let records = evidence["decisions"]["records"]
        .as_array()
        .expect("decision records");
    let record = records
        .iter()
        .find(|item| item["warp_decision"]["selected"]["command"]["command"] == "promote")
        .expect("selected promotion");
    let selected = &record["warp_decision"]["selected"];
    assert_eq!(selected["command"]["action_id"], action_id);
    assert_eq!(
        selected["command"]["grant"],
        serde_json::json!({
            "maximum_bytes": 8, "valid_until_ms": valid_until_ms,
        })
    );
    assert_eq!(
        selected["resources"],
        serde_json::json!({
            "network_bytes": 8, "storage_bytes": 8, "cpu_ms": 0, "requests": 0,
        })
    );
    assert_eq!(
        selected["authorized_resources"],
        serde_json::json!({
            "network_bytes": 4, "storage_bytes": 4, "cpu_ms": 0, "requests": 0,
        })
    );
    assert!(record.get("actual_resources").is_none());
}
