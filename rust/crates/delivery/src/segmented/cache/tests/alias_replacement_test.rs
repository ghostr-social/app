use crate::segmented::prepare::{PreparedHls, PreparedObject};
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

#[test]
fn replacing_cached_redirect_forgets_the_old_final_url_alias() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    cache.replace_focus(1, vec![(post.clone(), vec!["source".to_owned()])]);
    cache.complete(
        &post,
        1,
        Ok(prepared("source", "https://cdn.example/a", b"aaaa")),
    );
    assert!(cache.object("https://cdn.example/a").is_some());

    cache.complete(
        &post,
        1,
        Ok(prepared("source", "https://cdn.example/b", b"bbbb")),
    );
    assert!(cache.object("https://cdn.example/a").is_none());
    assert_eq!(
        cache.object("https://cdn.example/b").unwrap().body.as_ref(),
        b"bbbb"
    );
}

#[test]
fn exact_cache_key_takes_precedence_over_a_redirect_alias() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    cache.replace_focus(1, vec![(post.clone(), vec!["x".to_owned()])]);
    cache.complete(
        &post,
        1,
        Ok(prepared("x", "https://cdn.example/c", b"xxxx")),
    );
    cache.complete(
        &post,
        1,
        Ok(prepared(
            "https://cdn.example/c",
            "https://cdn.example/d",
            b"dddd",
        )),
    );

    let direct = cache.object("https://cdn.example/c").unwrap();
    assert_eq!(direct.final_url.as_str(), "https://cdn.example/d");
    assert_eq!(direct.body.as_ref(), b"dddd");
}

fn prepared(request_url: &str, final_url: &str, body: &[u8]) -> PreparedHls {
    PreparedHls {
        objects: vec![PreparedObject {
            request_url: request_url.to_owned(),
            final_url: Url::parse(final_url).unwrap(),
            body: Arc::from(body),
            content_type: None,
        }],
    }
}
