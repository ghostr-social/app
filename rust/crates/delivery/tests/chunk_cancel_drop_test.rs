use ghostr_delivery::chunk::cancel::cancel_pair;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn chunk_cancel_token_stays_uncancelled_when_the_handle_drops_silently() {
    let (handle, token) = cancel_pair();
    drop(handle);

    tokio::select! {
        _ = token.cancelled() => panic!("dropped handle must not cancel the token"),
        _ = tokio::time::sleep(Duration::from_secs(60)) => {}
    }
    assert!(!token.is_cancelled());
}
