use super::response;
use crate::hls::asset_request::AssetRangeRequest;
use axum::body::to_bytes;
use axum::http::header::{CONTENT_LENGTH, CONTENT_RANGE};
use axum::http::StatusCode;
use ghostr_delivery::segmented::CachedHlsObject;
use reqwest::Url;
use std::sync::Arc;

#[tokio::test]
async fn cached_assets_apply_typed_ranges_and_normalize_empty_416() {
    let partial = response(&object(), AssetRangeRequest::Bounded { start: 2, last: 5 })
        .expect("valid test fixture");
    assert_eq!(partial.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(partial.headers()[CONTENT_RANGE], "bytes 2-5/8");
    assert_eq!(
        to_bytes(partial.into_body(), 4)
            .await
            .expect("valid test fixture"),
        "cdef"
    );

    let suffix =
        response(&object(), AssetRangeRequest::Suffix { length: 2 }).expect("valid test fixture");
    assert_eq!(suffix.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        to_bytes(suffix.into_body(), 2)
            .await
            .expect("valid test fixture"),
        "gh"
    );

    let empty =
        response(&object(), AssetRangeRequest::Suffix { length: 0 }).expect("valid test fixture");
    assert_eq!(empty.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(empty.headers()[CONTENT_RANGE], "bytes */8");
    assert_eq!(empty.headers()[CONTENT_LENGTH], "0");
}

fn object() -> CachedHlsObject {
    CachedHlsObject::new(
        Arc::from(&b"abcdefgh"[..]),
        Url::parse("https://media.example/segment.m4s").expect("valid test fixture"),
        Some("video/mp4".to_owned()),
    )
}
