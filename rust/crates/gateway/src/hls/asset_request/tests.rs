use super::{AssetRangeRequest, ResolvedAssetRange};
use axum::http::header::RANGE;
use axum::http::{HeaderMap, HeaderValue};

#[test]
fn collects_one_range_and_resolves_every_supported_shape() {
    assert_eq!(collected(&[]), AssetRangeRequest::Full);
    assert_eq!(collected(&["bytes=5-2"]), AssetRangeRequest::Full);
    assert_eq!(collected(&["bytes=0-1,3-4"]), AssetRangeRequest::Full);
    assert_eq!(collected(&["items=0-3"]), AssetRangeRequest::Full);
    assert_eq!(
        collected(&["bytes=0-1", "bytes=2-3"]),
        AssetRangeRequest::Full
    );
    assert_eq!(
        collected(&["BYTES=4-7"]),
        AssetRangeRequest::Bounded { start: 4, last: 7 }
    );
    assert_eq!(
        collected(&["bytes=4-"]),
        AssetRangeRequest::Open { start: 4 }
    );
    let suffix = collected(&["bytes=-4"]);
    assert_eq!(suffix, AssetRangeRequest::Suffix { length: 4 });
    assert_eq!(suffix.resolve(16), partial(12, 16));
    assert_eq!(
        collected(&["bytes=-0"]).resolve(16),
        ResolvedAssetRange::Unsatisfiable
    );
}

fn collected(values: &[&str]) -> AssetRangeRequest {
    let mut headers = HeaderMap::new();
    for value in values {
        headers.append(RANGE, HeaderValue::from_str(value).expect("Range"));
    }
    AssetRangeRequest::collect(&headers)
}

fn partial(start: u64, end: u64) -> ResolvedAssetRange {
    ResolvedAssetRange::Partial { start, end }
}
