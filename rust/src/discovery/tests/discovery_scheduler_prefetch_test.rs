//! A hunger transition prefetches the active feed's next older page at
//! background priority, cursored one second below the oldest fetched
//! post (plan §5.4; discovery::pagination::next_page_cursor).

use super::scheduler_support::{
    context, next_outcome, next_started, note_at, request, start_scheduler,
};
use crate::discovery::retrieval_queue::RetrievalPriority;
use crate::engine::inventory_controller::Mode;
use crate::engine::DataUsageLevel;
use nostr_sdk::Timestamp;

#[tokio::test(start_paused = true)]
async fn hunger_prefetches_the_next_older_page() {
    let mut harness = start_scheduler(
        DataUsageLevel::Conservative,
        vec![note_at(100), note_at(90)],
    );
    harness.handle.open_feed(context("feed"), request());
    let first = next_started(&mut harness.started).await;
    assert_eq!(first.priority, RetrievalPriority::Interactive);

    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;
    harness
        .modes
        .send(Mode::Hunger)
        .expect("scheduler subscribed");

    let prefetch = next_started(&mut harness.started).await;
    assert_eq!(prefetch.context, context("feed"));
    assert_eq!(prefetch.priority, RetrievalPriority::Background);
    let primary = &prefetch.plan.queries[0].filter;
    assert_eq!(primary.until, Some(Timestamp::from(89)));
}
