//! Trending hashtags over recent posts, with deterministic ranking and a
//! 15-minute answer cache. Refreshes run as background discovery work.

use crate::event_parsing::ParsedVideoPost;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Most hashtags a trending answer carries.
pub const TRENDING_HASHTAG_LIMIT: usize = 12;

/// How long a computed answer keeps serving.
pub const TRENDING_TIME_TO_LIVE: Duration = Duration::from_secs(15 * 60);

/// Ranks hashtags by how many recent posts carry them; a tag repeated
/// inside one post counts once, and ties break alphabetically so the
/// ordering is stable across refreshes.
pub fn rank_trending_hashtags(posts: &[ParsedVideoPost], limit: usize) -> Vec<String> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for post in posts {
        let unique: HashSet<&str> = post.hashtags.iter().map(String::as_str).collect();
        for tag in unique {
            *counts.entry(tag).or_default() += 1;
        }
    }
    let mut ranked: Vec<&str> = counts.keys().copied().collect();
    ranked.sort_by(|left, right| {
        counts[right]
            .cmp(&counts[left])
            .then_with(|| left.cmp(right))
    });
    ranked.truncate(limit);
    ranked.into_iter().map(str::to_owned).collect()
}

/// Reuses a computed answer for [`TRENDING_TIME_TO_LIVE`]; the caller
/// injects `now` so expiry stays deterministic in tests.
#[derive(Debug, Default)]
pub struct TrendingHashtagsCache {
    cached: Option<(Vec<String>, Instant)>,
}

impl TrendingHashtagsCache {
    /// The cached answer while its age is strictly below the TTL.
    pub fn fresh(&self, now: Instant) -> Option<&[String]> {
        let (tags, stored_at) = self.cached.as_ref()?;
        if now.duration_since(*stored_at) < TRENDING_TIME_TO_LIVE {
            Some(tags.as_slice())
        } else {
            None
        }
    }

    pub fn store(&mut self, tags: Vec<String>, now: Instant) {
        self.cached = Some((tags, now));
    }
}
