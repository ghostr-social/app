use super::response;
use crate::hls::asset_request::AssetRangeRequest;
use ghostr_delivery::segmented::CachedHlsObject;
use reqwest::Url;
use std::sync::Arc;

#[test]
fn cached_response_keeps_the_cache_allocation_as_its_body_owner() {
    let body: Arc<[u8]> = Arc::from(vec![7; 1024]);
    let object = CachedHlsObject::new(
        body.clone(),
        Url::parse("https://media.example/segment.m4s").unwrap(),
        None,
    );

    let response = response(object, AssetRangeRequest::Full).unwrap();

    assert_eq!(Arc::strong_count(&body), 2);
    drop(response);
    assert_eq!(Arc::strong_count(&body), 1);
}
