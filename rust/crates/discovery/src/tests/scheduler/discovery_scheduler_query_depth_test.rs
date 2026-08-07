use crate::tests::scheduler_support::{context, next_outcome, next_started, note_at, request};
use crate::tests::scripted_scheduler_support::scripted_scheduler;
use crate::retrieval_types::RetrievalOutcome;
use nostr_sdk::{EventBuilder, Keys, Kind, Timestamp};

#[tokio::test]
async fn query_walks_raw_cursor_without_a_dart_or_mode_command() {
    let junk = EventBuilder::new(Kind::TextNote, "not playable")
        .custom_created_at(Timestamp::from(100))
        .sign_with_keys(&Keys::generate())
        .expect("signed");
    let mut harness = scripted_scheduler(vec![vec![junk], vec![note_at(50)]]);
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    let feed = context("search");

    harness.handle.open_feed(feed.clone(), query);
    let head = next_started(&mut harness.started).await;
    assert_eq!(head.plan.queries[0].filter.until, None);
    next_outcome(&mut harness.outcomes).await;

    let older = next_started(&mut harness.started).await;
    assert_eq!(
        older.plan.queries[0].filter.until,
        Some(Timestamp::from(99))
    );
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Started { .. }
    ));
    assert!(matches!(
        next_outcome(&mut harness.outcomes).await,
        RetrievalOutcome::Completed {
            result: Ok(events),
            ..
        } if events.len() == 1
    ));
    harness.handle.close_feed(feed);
}
