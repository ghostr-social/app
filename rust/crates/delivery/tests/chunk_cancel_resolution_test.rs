use core::time::Duration;
use ghostr_delivery::chunk::cancel::cancel_pair;

#[tokio::test(start_paused = true)]
async fn chunk_cancel_token_resolves_only_after_the_handle_cancels() {
    let (handle, token) = cancel_pair();
    assert!(!token.is_cancelled());

    let waiting = token.cancelled();
    tokio::pin!(waiting);
    tokio::select! {
        () = &mut waiting => panic!("token resolved before cancel"),
        () = tokio::time::sleep(Duration::from_millis(10)) => {}
    }

    handle.cancel();
    assert!(token.is_cancelled());
    waiting.await;
    token.cancelled().await;
}
