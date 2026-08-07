//! Older-page cursor math for Rust discovery queries.

use nostr_sdk::Timestamp;

/// Seconds stepped back from the oldest fetched post so the next page's
/// inclusive `until` cannot re-fetch that post.
pub const NEXT_PAGE_BACKSTEP_SECS: u64 = 1;

/// Inclusive `until` cutoff from a UTC unix-millisecond clock value.
pub fn older_than_from_unix_millis(millis: u64) -> Timestamp {
    Timestamp::from(millis / 1000)
}

/// Cursor for the page after fetching these posts: one second before the
/// oldest fetched `created_at`; `None` when nothing was fetched (the feed
/// is exhausted). Callers pass what was fetched, not what survived
/// filtering, so pages full of blocked creators cannot stall pagination.
pub fn next_page_cursor<I>(fetched_created_at: I) -> Option<Timestamp>
where
    I: IntoIterator<Item = Timestamp>,
{
    fetched_created_at
        .into_iter()
        .min()
        .map(|oldest| Timestamp::from(oldest.as_u64().saturating_sub(NEXT_PAGE_BACKSTEP_SECS)))
}
