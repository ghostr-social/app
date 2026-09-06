use crate::api::runtime::discovery::{lock, pump_outcomes};
use crate::api::tests::feed_pump_fixture::FeedPumpFixture;
use core::future::Future as _;
use core::pin::pin;
use core::task::{Context, Waker};

#[tokio::test]
async fn relay_backlog_yields_to_playback_between_bounded_bursts() {
    let fixture = FeedPumpFixture::new();
    fixture.queue_progress(64);
    let sinks = fixture.sinks();
    drop(fixture.sender);
    let mut pump = pin!(pump_outcomes(sinks, fixture.outcomes));
    let mut context = Context::from_waker(Waker::noop());

    assert!(
        pump.as_mut().poll(&mut context).is_pending(),
        "ready relay work must yield before monopolizing the playback executor"
    );
    let published = lock(&fixture.state).snapshot(fixture.feed).len();
    assert!(
        (1..=32).contains(&published),
        "make progress while bounding synchronous work: {published}"
    );

    pump.await;
    assert_eq!(lock(&fixture.state).snapshot(fixture.feed).len(), 64);
}
