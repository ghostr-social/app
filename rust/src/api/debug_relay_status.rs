//! Live relay status projection for the debug web page.

use ghostr_delivery::debug_feed::{DebugFeed, DebugRelaySnapshot};
use nostr_sdk::Client;
use std::sync::Arc;
use std::time::Duration;

pub(crate) async fn monitor(client: Arc<Client>, feed: DebugFeed, configured: Vec<String>) {
    loop {
        feed.update_relays(snapshot(&client, &configured).await);
        tokio::time::sleep(Duration::from_millis(750)).await;
    }
}

pub(crate) async fn snapshot(client: &Client, configured: &[String]) -> Vec<DebugRelaySnapshot> {
    let pool = client.relays().await;
    configured
        .iter()
        .map(|url| {
            let status = pool
                .iter()
                .find(|(relay_url, _)| relay_url.to_string() == *url)
                .map_or_else(
                    || "unavailable".to_owned(),
                    |(_, relay)| relay.status().to_string().to_ascii_lowercase(),
                );
            DebugRelaySnapshot {
                url: url.clone(),
                status,
            }
        })
        .collect()
}
