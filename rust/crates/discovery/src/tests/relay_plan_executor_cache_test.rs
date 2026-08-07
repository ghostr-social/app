//! The executor pool is separate from the client's seen-ID database, so
//! late relay bookkeeping cannot cross an account reset.

use crate::event_cache::{client_with_event_cache, ViewerScope};
use crate::relay_plan_executor::RelayPlanExecutor;
use crate::search_queries::plan_discovery;
use crate::tests::event_cache_support::{ids, note, notes};
use crate::tests::outbox_support::empty_directory;
use crate::tests::support::{author, AUTHOR_A, AUTHOR_B};
use crate::video_filters::DiscoveryRequest;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::prelude::*;
use std::sync::Arc;

fn executor(client: Arc<Client>) -> RelayPlanExecutor {
    RelayPlanExecutor::new(
        client,
        Vec::new(),
        empty_directory(),
        DataUsageLevel::Balanced,
    )
}

fn request(viewer: ViewerScope) -> DiscoveryRequest {
    DiscoveryRequest {
        viewer,
        ..DiscoveryRequest::default()
    }
}

#[tokio::test]
async fn client_seen_ids_are_not_rows_in_the_account_cache() {
    let client = Arc::new(client_with_event_cache());
    let executor = executor(client.clone());

    client
        .database()
        .save_event(&note(100))
        .await
        .expect("the client stores what it receives");

    assert!(ids(&executor.cache().stored(&notes()).await).is_empty());
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
