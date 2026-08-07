//! The trending answer is capped at the requested limit; the product
//! default is twelve tags.

use crate::tests::trending_support::post_with_hashtags;
use crate::query::trending::{rank_trending_hashtags, TRENDING_HASHTAG_LIMIT};

#[test]
fn default_limit_matches_dart() {
    assert_eq!(TRENDING_HASHTAG_LIMIT, 12);
}

#[test]
fn caps_the_ranking_at_the_limit() {
    let tags: Vec<String> = (0..13).map(|index| format!("tag{index:02}")).collect();
    let borrowed: Vec<&str> = tags.iter().map(String::as_str).collect();
    let posts = vec![post_with_hashtags("p1", &borrowed)];

    let ranked = rank_trending_hashtags(&posts, TRENDING_HASHTAG_LIMIT);

    assert_eq!(ranked.len(), 12);
    assert_eq!(ranked.first().map(String::as_str), Some("tag00"));
}

#[test]
fn honors_a_custom_limit() {
    let posts = vec![post_with_hashtags("p1", &["a", "b", "c"])];

    assert_eq!(rank_trending_hashtags(&posts, 2), vec!["a", "b"]);
}
