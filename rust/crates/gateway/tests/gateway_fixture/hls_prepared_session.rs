use axum::body::{to_bytes, Body};
use axum::http::Request;
use core::time::Duration;
use ghostr_delivery::segmented::{HlsPreparedAssetAuthority, SegmentedCache, SegmentedSnapshot};
use ghostr_gateway::hls::sessions::HlsSessionId;
use tower::ServiceExt as _;

#[derive(Debug, Eq, PartialEq)]
pub struct BootstrapBodies {
    pub init: String,
    pub segment: String,
}

pub async fn wait_authority(
    cache: &SegmentedCache,
    previous: Option<&HlsPreparedAssetAuthority>,
) -> HlsPreparedAssetAuthority {
    let changed = cache.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(authority) = current_authority(cache.snapshot("stream"), previous) {
                return authority;
            }
            notified.await;
        }
    })
    .await
    .expect("prepared HLS authority timeout")
}

pub async fn bootstrap(router: &axum::Router, session: &HlsSessionId) -> BootstrapBodies {
    let root = text(router, &format!("/hls/{}/index.m3u8", session.as_str())).await;
    let init_path = root
        .split("URI=\"")
        .nth(1)
        .and_then(|tail| tail.split('"').next())
        .expect("rewritten initialization path");
    let segment_path = root
        .lines()
        .find(|line| line.starts_with("/hls/") && line.contains("/assets/"))
        .expect("rewritten segment path");
    BootstrapBodies {
        init: text(router, init_path).await,
        segment: text(router, segment_path).await,
    }
}

fn current_authority(
    snapshot: SegmentedSnapshot,
    previous: Option<&HlsPreparedAssetAuthority>,
) -> Option<HlsPreparedAssetAuthority> {
    snapshot
        .authority
        .filter(|authority| previous != Some(authority))
}

async fn text(router: &axum::Router, path: &str) -> String {
    let response = router
        .clone()
        .oneshot(request(path))
        .await
        .expect("valid test fixture");
    assert!(response.status().is_success(), "{path} was not served");
    let bytes = to_bytes(response.into_body(), 4_096)
        .await
        .expect("bounded response body");
    String::from_utf8(bytes.to_vec()).expect("UTF-8 fixture body")
}

fn request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("valid test fixture")
}
