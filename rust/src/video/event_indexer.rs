use crate::video::event_index::NativeVideoIndex;
use log::warn;
use nostr_sdk::{Client, RelayPoolNotification};
use std::sync::Arc;

pub fn spawn_event_identity_indexer(client: Arc<Client>, index: NativeVideoIndex) {
    let notifications = client.notifications();
    tokio::spawn(run_event_identity_indexer(notifications, index));
}

pub async fn run_event_identity_indexer(
    mut notifications: tokio::sync::broadcast::Receiver<RelayPoolNotification>,
    index: NativeVideoIndex,
) {
    loop {
        match notifications.recv().await {
            Ok(RelayPoolNotification::Event { event, .. }) => index.record(&event).await,
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                warn!("Native event indexer skipped {skipped} notifications");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}
