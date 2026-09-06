use crate::api::runtime::discovery::{lock, pump_outcomes};
use crate::api::tests::feed_fixtures::video_note;
use crate::api::tests::feed_pump_fixture::FeedPumpFixture;
use nostr_sdk::Keys;

#[tokio::test]
async fn completion_rebases_before_subsequent_progress_in_the_same_ready_burst() {
    let fixture = FeedPumpFixture::new();
    let keys = Keys::generate();
    fixture.queue_event(video_note(&keys, "provisional", 10));
    fixture.queue_completion(vec![video_note(&keys, "complete", 40)]);
    fixture.queue_event(video_note(&keys, "later", 50));
    let sinks = fixture.sinks();
    drop(fixture.sender);

    pump_outcomes(sinks, fixture.outcomes).await;

    let timestamps: Vec<_> = lock(&fixture.state)
        .snapshot(fixture.feed)
        .iter()
        .map(|post| post.created_at)
        .collect();
    assert_eq!(
        timestamps,
        [50, 40],
        "completion remains an ordered rebase barrier"
    );
}
