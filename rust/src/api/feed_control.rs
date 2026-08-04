//! Feed lifecycle over the FFI (plan §2 phase-2 additions): open,
//! page, and close feeds assembled by Rust discovery.

use crate::api::feed_mapping::{parse_feed_id, parse_feed_spec};
use crate::api::feed_types::FfiFeedSpec;
use crate::api::runtime_registry;
use flutter_rust_bridge::frb;
use nostr_sdk::Timestamp;

/// Opens one feed in the Rust feed store, starts its first-page
/// queries, and returns the feed handle every later call names.
#[frb]
pub async fn ffi_open_feed(spec: FfiFeedSpec) -> anyhow::Result<String> {
    let parsed = parse_feed_spec(&spec)?;
    let engine = runtime_registry::engine()?;
    Ok(engine.discovery.open_feed(parsed))
}

/// Requests one older page. Returns whether more content may exist:
/// `false` once the feed is exhausted (or unknown), `true` while a
/// page is loading or a cursor remains. `older_than_secs` overrides
/// the tracked cursor when given.
#[frb]
pub async fn ffi_load_more(feed_id: String, older_than_secs: Option<u64>) -> anyhow::Result<bool> {
    let feed = parse_feed_id(&feed_id)?;
    let engine = runtime_registry::engine()?;
    Ok(engine
        .discovery
        .load_more(feed, older_than_secs.map(Timestamp::from)))
}

/// Closes the feed: its posts drop and its update streams end.
#[frb]
pub async fn ffi_close_feed(feed_id: String) -> anyhow::Result<()> {
    let feed = parse_feed_id(&feed_id)?;
    runtime_registry::engine()?.discovery.close_feed(feed);
    Ok(())
}
