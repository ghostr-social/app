//! Hashtags rank by how many recent posts carry them; ties break
//! alphabetically so the ordering is stable across refreshes — parity:
//! rankTrendingHashtags in
//! lib/features/video_catalog/domain/trending_hashtags.dart.

use super::trending_support::post_with_hashtags;
use crate::discovery::trending::{rank_trending_hashtags, TRENDING_HASHTAG_LIMIT};

#[test]
fn ranks_by_post_count_with_alphabetical_ties() {
    let posts = vec![
        post_with_hashtags("p1", &["music", "dance"]),
        post_with_hashtags("p2", &["dance"]),
        post_with_hashtags("p3", &["art", "music"]),
        post_with_hashtags("p4", &[]),
    ];

    let ranked = rank_trending_hashtags(&posts, TRENDING_HASHTAG_LIMIT);

    assert_eq!(ranked, vec!["dance", "music", "art"]);
}

#[test]
fn no_posts_yield_no_trending_hashtags() {
    assert!(rank_trending_hashtags(&[], TRENDING_HASHTAG_LIMIT).is_empty());
}
