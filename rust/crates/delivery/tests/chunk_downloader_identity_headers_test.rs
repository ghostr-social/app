mod range_fixture;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Response, StatusCode};
use axum::routing::get;
use axum::Router;
use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn continuation_requests_identity_bytes_with_if_range() {
    let seen = Arc::new(Mutex::new(None));
    let app = Router::new()
        .route("/video.mp4", get(reply))
        .with_state(seen.clone());
    let url = range_fixture::ranged::serve(app).await;
    let generation = SourceGeneration::try_new(&url, "\"version-one\"", 16).unwrap();
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("chunk-identity-headers");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        range: ByteRange::new(4, 8),
        continuation: Some(&generation),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    range_fixture::download_chunk_throttled(
        &spec,
        &sink,
        &mut HostStats::new(),
        &token,
        &range_fixture::network(),
    )
    .await
    .unwrap();

    let headers = seen.lock().unwrap().clone().unwrap();
    assert_eq!(
        headers,
        ("identity".to_owned(), "\"version-one\"".to_owned())
    );
    let _ = std::fs::remove_dir_all(root);
}

async fn reply(
    State(seen): State<Arc<Mutex<Option<(String, String)>>>>,
    headers: HeaderMap,
) -> Response<Body> {
    let encoding = header_value(&headers, header::ACCEPT_ENCODING);
    let if_range = header_value(&headers, header::IF_RANGE);
    *seen.lock().unwrap() = Some((encoding, if_range));
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_RANGE, "bytes 4-7/16")
        .header(header::ETAG, "\"version-one\"")
        .body(Body::from("4567"))
        .unwrap()
}

fn header_value(headers: &HeaderMap, name: header::HeaderName) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}
