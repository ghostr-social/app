use crate::catalog::{Catalog, LearnedFacts};
use crate::{DeliveryKind, PostId, VideoMeta};

#[test]
fn one_mirrors_range_response_does_not_poison_another_mirror() {
    let post = PostId::new("post");
    let first = "https://first.example/video.mp4";
    let second = "https://second.example/video.mp4";
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), meta(first, second));
    let identity = binding.transfer(first).unwrap();

    assert!(catalog.learn_response_for(
        &identity,
        LearnedFacts {
            accept_ranges: Some(false),
            ..LearnedFacts::default()
        },
    ));

    let entry = catalog.lookup(&post).unwrap();
    assert_eq!(entry.observed_range_support_for(first), Some(false));
    assert_eq!(entry.observed_range_support_for(second), None);
}

fn meta(first: &str, second: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![first.to_owned(), second.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: Some(1_000),
    }
}
