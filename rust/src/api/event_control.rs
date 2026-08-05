//! Generic Nostr reads over the shared Rust discovery engine.

use crate::api::event_types::{FfiNostrEvent, FfiNostrEventFilter};
use crate::api::runtime_registry;
use anyhow::{anyhow, Result};
use flutter_rust_bridge::frb;
use nostr_sdk::Filter;

const MAX_BATCH_FILTERS: usize = 20;

#[frb]
pub async fn ffi_query_events(filter: FfiNostrEventFilter) -> Result<Vec<FfiNostrEvent>> {
    query_filters(vec![filter]).await
}

#[frb]
pub async fn ffi_query_events_batch(
    filters: Vec<FfiNostrEventFilter>,
) -> Result<Vec<FfiNostrEvent>> {
    query_filters(filters).await
}

async fn query_filters(filters: Vec<FfiNostrEventFilter>) -> Result<Vec<FfiNostrEvent>> {
    anyhow::ensure!(
        filters.len() <= MAX_BATCH_FILTERS,
        "the query batch exceeds {MAX_BATCH_FILTERS} filters"
    );
    if filters.is_empty() {
        return Ok(Vec::new());
    }
    let filters = validated_filters(filters)?;
    let engine = runtime_registry::engine()?;
    let events = engine
        .discovery
        .query_events(filters)
        .await
        .map_err(|failure| anyhow!(failure.message))?;
    Ok(events.iter().map(FfiNostrEvent::from).collect())
}

fn validated_filters(filters: Vec<FfiNostrEventFilter>) -> Result<Vec<Filter>> {
    filters.into_iter().map(Filter::try_from).collect()
}
