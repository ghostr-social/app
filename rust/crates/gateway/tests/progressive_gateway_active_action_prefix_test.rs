mod gateway_fixture;

use axum::http::{header, StatusCode};
use core::time::Duration;
use gateway_fixture::progressive_action_response::ActionResponse;
use ghostr_engine::playback::PLAYBACK_SLICE_BYTES;
use tokio_stream::StreamExt as _;
use tower::ServiceExt as _;

#[tokio::test]
async fn active_action_prefix_streams_before_eof_and_stops_when_revoked() {
    let total = PLAYBACK_SLICE_BYTES * 2;
    let response = ActionResponse::new(total).await;
    let prefix = vec![b'a'; PLAYBACK_SLICE_BYTES as usize];
    response.write(0, &prefix).await;
    let request = response.gateway.video_request("clip", None).await;
    let reply = response
        .gateway
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("gateway response");
    assert_eq!(reply.status(), StatusCode::OK);
    assert_eq!(reply.headers()[header::CONTENT_LENGTH], total.to_string());
    let mut body = reply.into_body().into_data_stream();
    let first = tokio::time::timeout(Duration::from_secs(1), body.next())
        .await
        .expect("visible prefix before EOF")
        .expect("first chunk")
        .expect("prefix bytes");
    assert_eq!(first.as_ref(), prefix);
    assert!(!response
        .gateway
        .store
        .is_complete("clip")
        .await
        .expect("completion state"));

    response.action.revoke();
    response
        .gateway
        .store
        .release_action(&response.action)
        .await;
    let stopped = tokio::time::timeout(Duration::from_secs(1), body.next())
        .await
        .expect("revocation stops the old stream")
        .expect("terminal body error");
    assert!(stopped.is_err());
    std::fs::remove_dir_all(&response.gateway.root).expect("remove test store");
}
