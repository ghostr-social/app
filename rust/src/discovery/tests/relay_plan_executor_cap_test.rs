//! The executor resolves the main feed's discovery lookup through the
//! viewer's tracked follows and re-reads the data-usage cap on every
//! query, so `ffi_set_delivery_config` widens or narrows the fan-out of
//! feeds that are already open.

use crate::discovery::relay_plan_executor::RelayPlanExecutor;
use crate::discovery::search_queries::OutboxLookup;
use crate::discovery::tests::outbox_support::{directory_with_follows, shared_directory};
use crate::engine::DataUsageLevel;
use nostr_sdk::Client;
use std::sync::Arc;

fn executor() -> RelayPlanExecutor {
    RelayPlanExecutor::new(
        Arc::new(Client::default()),
        Vec::new(),
        shared_directory(directory_with_follows(24)),
        DataUsageLevel::Balanced,
    )
}

async fn discovery_relay_count(executor: &RelayPlanExecutor) -> usize {
    executor
        .outbox_relays(&OutboxLookup::DiscoveryRelays)
        .await
        .expect("a populated directory always resolves relays")
        .len()
}

#[tokio::test]
async fn the_live_data_usage_level_caps_the_resolved_relays() {
    let executor = executor();

    // One bootstrap relay plus the level's outbox cap (6/12/18).
    assert_eq!(discovery_relay_count(&executor).await, 13);

    executor.set_data_usage(DataUsageLevel::Conservative);
    assert_eq!(discovery_relay_count(&executor).await, 7);

    executor.set_data_usage(DataUsageLevel::Aggressive);
    assert_eq!(discovery_relay_count(&executor).await, 19);
}
