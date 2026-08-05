//! Runtime generic reads propagate scheduler failures without feed side effects.

use crate::api::tests::runtime_fixture::runtime;
use nostr_sdk::{Filter, Kind};

#[tokio::test]
async fn runtime_query_reports_that_no_relays_are_configured() {
    let error = runtime()
        .await
        .query_events(vec![Filter::new().kind(Kind::Reaction)])
        .await
        .expect_err("query should require a relay");

    assert_eq!(error.message, "no Nostr relays are configured");
}
