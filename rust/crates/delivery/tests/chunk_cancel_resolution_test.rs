use ghostr_delivery::chunk_cancel::cancel_pair;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn chunk_cancel_token_resolves_only_after_the_handle_cancels() {
    let (handle, token) = cancel_pair();
    assert!(!token.is_cancelled());

    let waiting = token.cancelled();
    tokio::pin!(waiting);
    tokio::select! {
        _ = &mut waiting => panic!("token resolved before cancel"),
        _ = tokio::time::sleep(Duration::from_millis(10)) => {}
    }

    handle.cancel();
    assert!(token.is_cancelled());
    waiting.await;
    token.cancelled().await;
}
