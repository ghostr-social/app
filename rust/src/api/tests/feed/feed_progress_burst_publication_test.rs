use crate::api::feed_types::FfiFeedStage;
use crate::api::runtime::discovery::{lock, pump_outcomes};
use crate::api::tests::feed_pump_fixture::FeedPumpFixture;

#[tokio::test]
async fn a_ready_relay_burst_publishes_bounded_updates_without_waiting_for_completion() {
    let fixture = FeedPumpFixture::new();
    fixture.queue_progress(64);
    let sinks = fixture.sinks();
    drop(fixture.sender);

    pump_outcomes(sinks, fixture.outcomes).await;

    assert_eq!(lock(&fixture.state).snapshot(fixture.feed).len(), 64);
    assert_eq!(
        lock(&fixture.state).stage(fixture.feed),
        FfiFeedStage::Loading
    );
    let publications = *fixture.revisions.borrow();
    assert!(
        (1..=4).contains(&publications),
        "64 ready posts must not rebuild and publish the full feed individually: {publications}"
    );
}
