mod delivery_fixture;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Method, StatusCode, Uri};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use core::sync::atomic::{AtomicUsize, Ordering};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{DataUsageLevel, EngineParams};
use std::sync::Arc;
use tokio::net::TcpListener;

const BODY_LEN: usize = 293_999;
type OriginState = (Arc<Vec<u8>>, Arc<AtomicUsize>);

#[tokio::test]
async fn cold_zero_start_range_promotes_one_live_200_without_restart() {
    let body = vec![7; BODY_LEN];
    let (origin, gets) = serve_range_blind(body.clone()).await;
    let options = DeliveryOptions {
        level: DataUsageLevel::Aggressive,
        params: EngineParams::default(),
        ..DeliveryOptions::default()
    };
    let harness = start_harness("cold-range-promotion", options);
    let items = vec![
        sized_item(
            "current",
            &format!("{origin}/current.mp4"),
            BODY_LEN as u64,
            6_000,
        ),
        sized_item(
            "future-one",
            &format!("{origin}/future-one.mp4"),
            BODY_LEN as u64,
            6_000,
        ),
        sized_item(
            "future-two",
            &format!("{origin}/future-two.mp4"),
            BODY_LEN as u64,
            6_000,
        ),
    ];

    harness.handle.update_focus(focus_now(items, 0, 5_000));
    wait_for_ranges(&harness.store, "current", &[(0, BODY_LEN as u64)]).await;

    assert_eq!(gets.load(Ordering::SeqCst), 1, "live 200 was restarted");
    assert_eq!(
        harness
            .store
            .read_range("current", 0..BODY_LEN as u64)
            .await
            .expect("read"),
        Some(body)
    );
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}

async fn serve_range_blind(body: Vec<u8>) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let gets = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/{video}", get(range_blind).head(range_blind))
        .with_state((Arc::new(body), Arc::clone(&gets)));
    tokio::spawn(async move { axum::serve(listener, app).await.expect("origin") });
    (format!("http://{address}"), gets)
}

async fn range_blind(
    State((body, gets)): State<OriginState>,
    method: Method,
    uri: Uri,
) -> Response {
    if method == Method::HEAD {
        return core::future::pending().await;
    }
    if uri.path() == "/current.mp4" {
        gets.fetch_add(1, Ordering::SeqCst);
    }
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, "\"cold-range-promotion\"")
        .body(Body::from(body.to_vec()))
        .expect("response")
}
