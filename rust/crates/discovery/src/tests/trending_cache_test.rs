//! A computed trending answer serves for 15 minutes and expires exactly
//! at the time-to-live.

use crate::trending::{TrendingHashtagsCache, TRENDING_TIME_TO_LIVE};
use std::time::{Duration, Instant};

#[test]
fn serves_within_the_ttl_and_expires_at_it() {
    let mut cache = TrendingHashtagsCache::default();
    let start = Instant::now();
    assert!(cache.fresh(start).is_none(), "an empty cache holds nothing");

    cache.store(vec!["dance".to_owned()], start);

    let almost_stale = start + TRENDING_TIME_TO_LIVE - Duration::from_secs(1);
    let expected = ["dance".to_owned()];
    assert_eq!(cache.fresh(almost_stale), Some(&expected[..]));
    assert!(
        cache.fresh(start + TRENDING_TIME_TO_LIVE).is_none(),
        "the answer must expire at the ttl, matching Dart's strict <",
    );
}

#[test]
fn ttl_matches_dart() {
    assert_eq!(TRENDING_TIME_TO_LIVE, Duration::from_secs(15 * 60));
}
