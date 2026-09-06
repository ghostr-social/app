use crate::api::runtime::discovery::{lock, pump_outcomes};
use crate::api::tests::feed_pump_fixture::FeedPumpFixture;
use crate::discovery::feed::spec::FeedSpec;
use core::future::Future as _;
use core::pin::pin;
use core::task::{Context, Waker};

#[tokio::test]
async fn a_session_reset_fences_progress_waiting_behind_a_yield() {
    let fixture = FeedPumpFixture::new();
    fixture.queue_progress(64);
    let sinks = fixture.sinks();
    drop(fixture.sender);
    let mut pump = pin!(pump_outcomes(sinks, fixture.outcomes));
    assert!(
        pump.as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending(),
        "the first bounded burst yields"
    );
    assert!(
        !lock(&fixture.state).snapshot(fixture.feed).is_empty(),
        "first burst arrived"
    );

    lock(&fixture.state).reset_session();
    let (fresh, _) = lock(&fixture.state).open(FeedSpec::Search("clip".to_owned()));
    pump.await;

    assert!(
        lock(&fixture.state).snapshot(fresh).is_empty(),
        "old progress cannot enter a fresh session"
    );
    assert!(
        lock(&fixture.state).snapshot(fixture.feed).is_empty(),
        "closed feed stays closed"
    );
}
