use crate::execution::collector::collect_page;
use crate::execution::fetch::FetchedEvents;
use crate::query::search::QueryRole;
use crate::retrieval_types::PlanFailure;
use nostr_sdk::{EventBuilder, Keys};

#[tokio::test]
async fn additive_failure_keeps_the_successful_page_unsettled() {
    let event = EventBuilder::text_note("primary")
        .sign_with_keys(&Keys::generate())
        .unwrap();
    let primary = tokio::spawn({
        let event = event.clone();
        async move { Ok(FetchedEvents::fresh(vec![event])) }
    });
    let additive = tokio::spawn(async { Err(PlanFailure::new("additive failed")) });

    let page = collect_page(vec![
        (QueryRole::Primary, primary),
        (QueryRole::Additive, additive),
    ])
    .await
    .expect("safe results remain usable");

    assert_eq!(page.events, vec![event]);
    assert_eq!(page.cursor, None);
    assert!(!page.complete);
}
