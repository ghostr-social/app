use crate::catalog::{Catalog, LearnedFacts};
use crate::{DeliveryKind, PostId, VideoMeta};

#[test]
fn stale_source_facts_cannot_cross_a_representation_generation() {
    let post = PostId::new("same");
    let mut catalog = Catalog::new();
    let first = catalog.upsert(post.clone(), meta("https://a.example/video", 8));
    let old = catalog
        .transfer_identity(&post, "https://a.example/video")
        .expect("first source identity");

    let unchanged = catalog.upsert(post.clone(), meta("https://a.example/video", 8));
    assert_eq!(unchanged, first);

    let second = catalog.upsert(post.clone(), meta("https://b.example/video", 4));
    assert_ne!(second, first);
    assert!(!catalog.learn_for(&old, learned(99)));
    assert_eq!(catalog.lookup(&post).unwrap().total_bytes(), Some(4));

    let current = catalog
        .transfer_identity(&post, "https://b.example/video")
        .expect("replacement source identity");
    assert!(catalog.learn_for(&current, learned(5)));
    assert_eq!(catalog.lookup(&post).unwrap().total_bytes(), Some(5));
}

#[test]
fn evicted_transfer_identity_cannot_reintroduce_learned_state() {
    let post = PostId::new("evicted");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta("https://a.example/video", 8));
    let evicted = catalog
        .transfer_identity(&post, "https://a.example/video")
        .expect("catalogued transfer identity");

    catalog.retain(|known| known != &post);

    assert!(!catalog.learn_for(&evicted, learned(99)));
    assert!(catalog.lookup(&post).is_none());
}

#[test]
fn verified_bytes_survive_mirror_rotation_but_not_a_delivery_kind_change() {
    let post = PostId::new("verified");
    let mut catalog = Catalog::new();
    let first = catalog.upsert(
        post.clone(),
        verified_meta("https://a.example/video", DeliveryKind::Progressive),
    );
    let fingerprint = first.representation().fingerprint();
    assert_eq!(first.post(), &post);

    let mirror = catalog.upsert(
        post.clone(),
        verified_meta("https://b.example/video", DeliveryKind::Progressive),
    );
    assert_eq!(mirror.representation().fingerprint(), fingerprint);
    assert_ne!(mirror, first);

    let hls = catalog.upsert(
        post,
        verified_meta("https://b.example/video", DeliveryKind::Hls),
    );
    assert_ne!(hls.representation().fingerprint(), fingerprint);
}

fn meta(url: &str, size: u64) -> VideoMeta {
    VideoMeta {
        urls: vec![url.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(size),
        duration_ms: Some(1_000),
    }
}

fn learned(size: u64) -> LearnedFacts {
    LearnedFacts {
        content_length: Some(size),
        accept_ranges: Some(true),
        host: Some("source.example".to_owned()),
    }
}

fn verified_meta(url: &str, delivery: DeliveryKind) -> VideoMeta {
    VideoMeta {
        urls: vec![url.to_owned()],
        delivery,
        sha256: Some("trusted-content-digest".to_owned()),
        size_bytes: None,
        duration_ms: None,
    }
}
