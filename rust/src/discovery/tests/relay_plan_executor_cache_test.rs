//! The executor's session pool is the shared client's own database, so
//! every event the relay layer stores on receipt (nostr-relay-pool
//! 0.38 relay/inner.rs saves each accepted event) is a row the next
//! query can answer with — the pool grows as the viewer browses. It is
//! scoped before any query reads it.

use crate::discovery::event_cache::{client_with_event_cache, ViewerScope};
use crate::discovery::relay_plan_executor::RelayPlanExecutor;
use crate::discovery::search_queries::plan_discovery;
use crate::discovery::tests::event_cache_support::{ids, note, notes};
use crate::discovery::tests::outbox_support::empty_directory;
use crate::discovery::tests::support::{author, AUTHOR_A, AUTHOR_B};
use crate::discovery::video_filters::DiscoveryRequest;
use crate::engine::DataUsageLevel;
use nostr_sdk::prelude::*;
use std::sync::Arc;

fn executor(client: Arc<Client>) -> RelayPlanExecutor {
    RelayPlanExecutor::new(client, Vec::new(), empty_directory(), DataUsageLevel::Balanced)
}

fn request(viewer: ViewerScope) -> DiscoveryRequest {
    DiscoveryRequest {
        viewer,
        ..DiscoveryRequest::default()
    }
}

#[tokio::test]
async fn the_pool_the_executor_reads_is_the_clients_own_database() {
    let client = Arc::new(client_with_event_cache());
    let executor = executor(client.clone());

    client
        .database()
        .save_event(&note(100))
        .await
        .expect("the client stores what it receives");

    assert_eq!(
        ids(&executor.cache().stored(&notes()).await),
        ids(&[note(100)])
    );
}

#[tokio::test]
async fn a_plan_naming_a_new_viewer_empties_the_pool_before_it_queries() {
    let executor = executor(Arc::new(client_with_event_cache()));
    let signed_in = plan_discovery(&request(ViewerScope::SignedIn(author(AUTHOR_A))));
    executor.adopt_plan_viewer(&signed_in).await;
    executor.cache().remember(&[note(100)]).await;

    let other = plan_discovery(&request(ViewerScope::SignedIn(author(AUTHOR_B))));

    assert!(executor.adopt_plan_viewer(&other).await);
    assert!(executor.cache().stored(&notes()).await.is_empty());
}
