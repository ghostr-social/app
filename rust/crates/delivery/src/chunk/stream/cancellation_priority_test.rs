use super::select_next;
use crate::chunk::cancel::cancel_pair;

#[tokio::test]
async fn ready_cancellation_always_wins_over_a_buffered_body_chunk() {
    for _ in 0..64 {
        let (handle, token) = cancel_pair();
        handle.cancel();
        let body = async { Ok(Some(bytes::Bytes::from_static(b"late"))) };

        assert_eq!(select_next(&token, body).await.expect("selection"), None);
    }
}
