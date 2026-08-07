//! Feed lifecycle over the FFI (plan §2 phase-2 additions): open,
//! page, and close feeds assembled by Rust discovery.

use crate::api::feed::mapping::{parse_feed_id, parse_feed_spec};
use crate::api::feed_types::FfiFeedSpec;
use crate::api::runtime::registry;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::session_generation::SessionGeneration;
use anyhow::ensure;
use flutter_rust_bridge::frb;
use nostr_sdk::{PublicKey, Timestamp};

/// Captures the native account-session token before Dart waits on any
/// previous feed. A later open must present the same token.
#[frb]
pub async fn ffi_feed_session(expected_account_hex: Option<String>) -> anyhow::Result<u64> {
    let expected_account = parse_expected_account(expected_account_hex)?;
    let engine = registry::engine()?;
    Ok(engine
        .discovery
        .feed_session(expected_account)
        .await?
        .value())
}

/// Opens one feed in the Rust feed store, starts its first-page
/// queries, and returns the feed handle every later call names.
#[frb]
pub async fn ffi_open_feed(
    spec: FfiFeedSpec,
    expected_account_hex: Option<String>,
    expected_session_generation: u64,
) -> anyhow::Result<String> {
    let parsed = parse_feed_spec(&spec)?;
    let expected_account = parse_expected_account(expected_account_hex)?;
    validate_main_account(&parsed, expected_account)?;
    let engine = registry::engine()?;
    let expected_session = SessionGeneration::from_value(expected_session_generation);
    engine
        .discovery
        .open_feed(parsed, expected_account, expected_session)
        .await
}

/// Requests one older page. Returns whether more content may exist:
/// `false` once the feed is exhausted (or unknown), `true` while a
/// page is loading or a cursor remains. `older_than_secs` overrides
/// the tracked cursor when given.
#[frb]
pub async fn ffi_load_more(feed_id: String, older_than_secs: Option<u64>) -> anyhow::Result<bool> {
    let feed = parse_feed_id(&feed_id)?;
    let engine = registry::engine()?;
    Ok(engine
        .discovery
        .load_more(feed, older_than_secs.map(Timestamp::from)))
}

/// Closes the feed: its posts drop and its update streams end.
#[frb]
pub async fn ffi_close_feed(feed_id: String) -> anyhow::Result<()> {
    let feed = parse_feed_id(&feed_id)?;
    registry::engine()?.discovery.close_feed(feed);
    Ok(())
}

fn parse_expected_account(raw: Option<String>) -> anyhow::Result<Option<PublicKey>> {
    Ok(raw.map(|value| PublicKey::from_hex(&value)).transpose()?)
}

fn validate_main_account(
    spec: &FeedSpec,
    expected_account: Option<PublicKey>,
) -> anyhow::Result<()> {
    if let FeedSpec::MainFeed { viewer } = spec {
        ensure!(
            viewer.as_ref() == expected_account.as_ref(),
            "the main feed viewer does not match the expected account"
        );
    }
    Ok(())
}
