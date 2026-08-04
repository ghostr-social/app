use crate::engine::catalog::{Catalog, LearnedFacts};
use crate::engine::tests::support::progressive_meta;
use crate::engine::PostId;

#[test]
fn lookup_returns_inserted_meta_with_blank_facts() {
    let mut catalog = Catalog::new();
    let post = PostId::new("a");

    catalog.upsert(post.clone(), progressive_meta(Some(100), None));

    let entry = catalog.lookup(&post).expect("entry");
    assert_eq!(entry.meta, progressive_meta(Some(100), None));
    assert_eq!(entry.facts, LearnedFacts::default());
    assert_eq!(catalog.len(), 1);
    assert!(!catalog.is_empty());
}

#[test]
fn unknown_posts_have_no_entry() {
    let catalog = Catalog::new();

    assert!(catalog.lookup(&PostId::new("missing")).is_none());
    assert!(catalog.is_empty());
}

#[test]
fn upsert_replaces_meta_but_keeps_learned_facts() {
    let mut catalog = Catalog::new();
    let post = PostId::new("a");
    catalog.upsert(post.clone(), progressive_meta(None, None));
    let learned = LearnedFacts {
        content_length: Some(42),
        accept_ranges: Some(true),
        host: Some("host.example".to_owned()),
    };
    assert!(catalog.learn(&post, learned.clone()));

    catalog.upsert(post.clone(), progressive_meta(Some(7), Some(1_000)));

    let entry = catalog.lookup(&post).expect("entry");
    assert_eq!(entry.meta, progressive_meta(Some(7), Some(1_000)));
    assert_eq!(entry.facts, learned);
    assert_eq!(catalog.len(), 1);
}

#[test]
fn learning_about_an_unknown_post_is_rejected() {
    let mut catalog = Catalog::new();

    assert!(!catalog.learn(&PostId::new("missing"), LearnedFacts::default()));
    assert!(catalog.is_empty());
}
