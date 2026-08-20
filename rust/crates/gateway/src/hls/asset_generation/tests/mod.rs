mod cache_binding_test;
mod origin_identity_test;

use super::origin::OriginGeneration;
use ghostr_net::strong_etag::single_strong_etag;
use reqwest::header::{HeaderMap, HeaderValue, ETAG};
use reqwest::Url;

fn origin_generation(url: &Url, etag: &str, total: u64) -> OriginGeneration {
    let mut headers = HeaderMap::new();
    headers.insert(ETAG, HeaderValue::from_str(etag).unwrap());
    let etag = single_strong_etag(&headers).unwrap().unwrap();
    OriginGeneration::new(url, etag, total)
}
