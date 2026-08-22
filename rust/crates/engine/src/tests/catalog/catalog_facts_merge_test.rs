use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::support::progressive_meta;
use crate::PostId;

fn catalog_with_post(post: &PostId) -> Catalog {
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), progressive_meta(Some(500), None));
    catalog
}

#[test]
fn later_lessons_overwrite_only_freshly_learned_fields() {
    let post = PostId::new("a");
    let mut catalog = catalog_with_post(&post);
    catalog.learn(
        &post,
        LearnedFacts {
            content_length: Some(42),
            accept_ranges: Some(false),
            host: Some("first.example".to_owned()),
        },
    );

    catalog.learn(
        &post,
        LearnedFacts {
            content_length: None,
            accept_ranges: Some(true),
            host: None,
        },
    );

    let facts = catalog
        .lookup(&post)
        .expect("entry")
        .observed_facts_for("https://host.example/video.mp4")
        .expect("response facts");
    assert_eq!(facts.content_length, Some(42));
    assert_eq!(facts.accept_ranges, Some(true));
    assert_eq!(facts.host.as_deref(), Some("first.example"));
}

#[test]
fn total_bytes_prefers_probed_content_length_over_imeta_size() {
    let post = PostId::new("a");
    let mut catalog = catalog_with_post(&post);
    assert_eq!(
        catalog.lookup(&post).expect("entry").total_bytes(),
        Some(500)
    );

    catalog.learn(
        &post,
        LearnedFacts {
            content_length: Some(777),
            ..LearnedFacts::default()
        },
    );

    assert_eq!(
        catalog.lookup(&post).expect("entry").total_bytes(),
        Some(777)
    );
}

#[test]
fn total_bytes_is_unknown_without_size_or_probe() {
    let post = PostId::new("a");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), progressive_meta(None, None));

    assert_eq!(catalog.lookup(&post).expect("entry").total_bytes(), None);
}
