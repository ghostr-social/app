use super::store_ready;
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

#[test]
fn cached_hls_generation_binds_final_url_length_and_bytes() {
    let baseline = generation("https://cdn.example/segment.m4s", b"same");
    assert_eq!(
        baseline,
        generation("https://cdn.example/segment.m4s", b"same")
    );
    assert_ne!(
        baseline,
        generation("https://cdn.example/segment.m4s", b"swap")
    );
    assert_ne!(
        baseline,
        generation("https://cdn.example/segment.m4s", b"same!")
    );
    assert_ne!(
        baseline,
        generation("https://other.example/segment.m4s", b"same")
    );
    assert_ne!(
        generation("https://cdn.example/a", b"bc"),
        generation("https://cdn.example/ab", b"c")
    );
}

fn generation(final_url: &str, body: &[u8]) -> crate::segmented::CachedHlsGeneration {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    cache.replace_focus(1, vec![(post.clone(), vec!["source".to_owned()])]);
    store_ready(&cache, &post, 1, vec![prepared(final_url, body)]);
    cache.object("source").expect("cached object").generation()
}

fn prepared(final_url: &str, body: &[u8]) -> PreparedObject {
    PreparedObject {
        request_url: "source".to_owned(),
        final_url: Url::parse(final_url).expect("final URL"),
        body: Arc::from(body),
        content_type: None,
    }
}
