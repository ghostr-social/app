use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn representation_change_keeps_a_live_probe_occupied_until_completion() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), unknown_meta("old"));
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    assert_eq!(
        probes
            .claim(&catalog, std::slice::from_ref(&post), &retry)
            .len(),
        1
    );

    catalog.upsert(post.clone(), unknown_meta("new"));
    probes.representation_changed(&post);
    assert!(probes
        .claim(&catalog, std::slice::from_ref(&post), &retry)
        .is_empty());

    probes.release(&post);
    assert_eq!(probes.claim(&catalog, &[post], &retry).len(), 1);
}

#[test]
fn manager_reset_keeps_a_live_probe_occupied_until_completion() {
    let first = PostId::new("first");
    let second = PostId::new("second");
    let mut catalog = Catalog::new();
    catalog.upsert(first.clone(), unknown_meta("first"));
    catalog.upsert(second.clone(), unknown_meta("second"));
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    assert_eq!(
        probes
            .claim(&catalog, std::slice::from_ref(&first), &retry)
            .len(),
        1
    );

    probes.clear();
    assert!(probes
        .claim(&catalog, std::slice::from_ref(&second), &retry)
        .is_empty());

    probes.release(&first);
    assert_eq!(probes.claim(&catalog, &[second], &retry).len(), 1);
}

#[test]
fn manager_reset_invalidates_a_live_probe_even_when_identity_numbers_restart() {
    let post = PostId::new("post");
    let meta = unknown_meta("same");
    let source = meta.urls[0].clone();
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta.clone());
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    assert_eq!(
        probes
            .claim(&catalog, std::slice::from_ref(&post), &retry)
            .len(),
        1
    );

    probes.clear();
    catalog = Catalog::new();
    catalog.upsert(post.clone(), meta);

    assert!(probes.current_identity(&catalog, &post, &source).is_none());
    assert!(probes
        .claim(&catalog, std::slice::from_ref(&post), &retry)
        .is_empty());
    probes.release(&post);
    assert_eq!(probes.claim(&catalog, &[post], &retry).len(), 1);
}

fn unknown_meta(id: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
