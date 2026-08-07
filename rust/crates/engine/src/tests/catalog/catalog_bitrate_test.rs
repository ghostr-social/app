use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::support::progressive_meta;
use crate::{EngineParams, PostId};

fn estimate(size: Option<u64>, duration_ms: Option<u64>) -> u64 {
    let mut catalog = Catalog::new();
    let post = PostId::new("a");
    catalog.upsert(post.clone(), progressive_meta(size, duration_ms));
    catalog.estimated_bitrate(&post, &EngineParams::default())
}

#[test]
fn measured_bitrate_comes_from_size_and_duration() {
    // 1_000_000 bytes over 4 s = 2_000_000 bits per second.
    assert_eq!(estimate(Some(1_000_000), Some(4_000)), 2_000_000);
}

#[test]
fn assumed_bitrate_covers_missing_or_degenerate_metadata() {
    let assumed = EngineParams::default().assumed_bitrate_bps;
    let cases = [
        (None, Some(4_000)),
        (Some(1_000_000), None),
        (Some(1_000_000), Some(0)),
        (None, None),
    ];

    for (size, duration) in cases {
        assert_eq!(estimate(size, duration), assumed, "{size:?}/{duration:?}");
    }
}

#[test]
fn unknown_posts_fall_back_to_the_assumed_bitrate() {
    let params = EngineParams::default();
    let catalog = Catalog::new();

    let bitrate = catalog.estimated_bitrate(&PostId::new("missing"), &params);

    assert_eq!(bitrate, params.assumed_bitrate_bps);
}

#[test]
fn probed_content_length_refines_the_estimate() {
    let mut catalog = Catalog::new();
    let post = PostId::new("a");
    catalog.upsert(post.clone(), progressive_meta(Some(1_000_000), Some(4_000)));
    catalog.learn(
        &post,
        LearnedFacts {
            content_length: Some(2_000_000),
            ..LearnedFacts::default()
        },
    );

    let bitrate = catalog.estimated_bitrate(&post, &EngineParams::default());

    assert_eq!(bitrate, 4_000_000);
}
