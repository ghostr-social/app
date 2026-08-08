//! Turning an intent — a feed, a search, a hashtag — into the Nostr
//! filters that answer it. Pure filter construction: nothing here
//! touches a relay or a cache.

pub mod events;
pub(crate) mod hashtags;
pub mod live_search_relays;
pub mod search;
pub mod trending;
pub mod video_filters;
