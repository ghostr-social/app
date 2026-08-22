use super::store_ready;
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

#[test]
fn replacing_source_forgets_the_old_final_url_alias() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    cache.replace_focus(1, vec![(post.clone(), vec!["source".to_owned()])]);
    store_ready(
        &cache,
        &post,
        1,
        vec![prepared("source", "https://cdn.example/a", b"aaaa")],
    );
    assert!(cache.object("https://cdn.example/a").is_some());

    cache.replace_focus(2, vec![(post.clone(), vec!["replacement".to_owned()])]);
    store_ready(
        &cache,
        &post,
        2,
        vec![prepared("replacement", "https://cdn.example/b", b"bbbb")],
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
    let other = PostId::new("other");
    cache.replace_focus(
        1,
        vec![
            (post.clone(), vec!["x".to_owned()]),
            (other.clone(), vec!["https://cdn.example/c".to_owned()]),
        ],
    );
    store_ready(
        &cache,
        &post,
        1,
        vec![prepared("x", "https://cdn.example/c", b"xxxx")],
    );
    store_ready(
        &cache,
        &other,
        1,
        vec![prepared(
            "https://cdn.example/c",
            "https://cdn.example/d",
            b"dddd",
        )],
    );

    let direct = cache.object("https://cdn.example/c").unwrap();
    assert_eq!(direct.final_url.as_str(), "https://cdn.example/d");
    assert_eq!(direct.body.as_ref(), b"dddd");
}

fn prepared(request_url: &str, final_url: &str, body: &[u8]) -> PreparedObject {
    PreparedObject {
        request_url: request_url.to_owned(),
        final_url: Url::parse(final_url).unwrap(),
        body: Arc::from(body),
        content_type: None,
        cache: Default::default(),
    }
}
