//! Auto-prefetch honors how far the capped wire filter actually reached.

use crate::tests::scheduler_support::{
    context, next_outcome, next_started, request, start_scheduler,
};
use ghostr_engine::{adaptive::DiscoveryDemand, DataUsageLevel};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};

fn note(content: &str, created_at: u64) -> Event {
    EventBuilder::new(Kind::TextNote, content)
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed note")
}

#[tokio::test(start_paused = true)]
async fn non_video_event_advances_the_complete_note_filter_cursor() {
    let events = vec![
        note("https://cdn.example/clip.mp4", 100),
        note("ordinary old note", 1),
    ];
    let mut harness = start_scheduler(DataUsageLevel::Conservative, events);
    harness.handle.open_feed(context("feed"), request());
    next_started(&mut harness.started).await;
    harness.gate.add_permits(1);
    next_outcome(&mut harness.outcomes).await;

    harness
        .demand
        .send(DiscoveryDemand::Expand)
        .expect("scheduler subscribed");

    let prefetch = next_started(&mut harness.started).await;
    assert_eq!(
        prefetch.plan.queries[0].filter.until,
        Some(Timestamp::from(0)),
    );
}
