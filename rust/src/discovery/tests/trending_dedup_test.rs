//! A hashtag repeated inside one post counts once — parity: the
//! per-post `toSet()` in rankTrendingHashtags
//! (lib/features/video_catalog/domain/trending_hashtags.dart).

use super::trending_support::post_with_hashtags;
use crate::discovery::trending::{rank_trending_hashtags, TRENDING_HASHTAG_LIMIT};

#[test]
fn repeated_tag_in_one_post_counts_once() {
    let posts = vec![
        post_with_hashtags("p1", &["zebra", "zebra", "zebra"]),
        post_with_hashtags("p2", &["art"]),
    ];

    let ranked = rank_trending_hashtags(&posts, TRENDING_HASHTAG_LIMIT);

    assert_eq!(ranked, vec!["art", "zebra"]);
}
