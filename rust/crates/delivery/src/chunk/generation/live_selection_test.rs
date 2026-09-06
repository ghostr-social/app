use super::OriginGeneration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::media_retention::MediaRetention;
use ghostr_net::outbound_media_client::MediaHttpClient;
use reqwest::header::{ACCEPT_ENCODING, RANGE};
use std::sync::Arc;

#[tokio::test]
#[ignore = "reads a real public media origin; run explicitly with network access"]
async fn live_origin_vary_preserves_a_usable_selected_sparse_generation() {
    let executor = MediaRequestExecutor::new(
        Arc::new(MediaHttpClient::public().expect("production client")),
        MediaRequestLimits::try_new(1, 1).expect("limits"),
    );
    let mut response = executor
        .get(
            "https://media.libernet.app/s/Z7sZ0T.mp4",
            PreemptionAuthority::PlaybackCritical,
        )
        .expect("real media URL")
        .header(ACCEPT_ENCODING, "identity".parse().expect("identity"))
        .header(RANGE, "bytes=0-0".parse().expect("range"))
        .admit()
        .await
        .expect("broker admission")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(20),
        )
        .await
        .expect("real origin");
    assert_eq!(response.status(), reqwest::StatusCode::PARTIAL_CONTENT);
    let generation = OriginGeneration::from_response(&response, Some(3_693_628));
    assert_eq!(generation.retention(), MediaRetention::Partitioned);
    let selected = generation.strict().expect("selected sparse generation");
    assert_eq!(selected.request_selection(), response.request_selection());
    assert!(selected.request_selection().is_some());
    assert_eq!(
        response
            .chunk()
            .await
            .expect("real bytes")
            .expect("body")
            .as_ref(),
        &[0]
    );
}
