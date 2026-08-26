use super::origin_generation;
use reqwest::Url;

#[test]
fn origin_generation_identity_includes_final_url_etag_and_total() {
    let url = Url::parse("https://media.example/segment.m4s").expect("valid test fixture");
    let baseline = origin_generation(&url, "\"v1\"", 16);
    assert_ne!(baseline, origin_generation(&url, "\"v2\"", 16));
    assert_ne!(baseline, origin_generation(&url, "\"v1\"", 20));
    let other = Url::parse("https://other.example/segment.m4s").expect("valid test fixture");
    assert_ne!(baseline, origin_generation(&other, "\"v1\"", 16));
}
