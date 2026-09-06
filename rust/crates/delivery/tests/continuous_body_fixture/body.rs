use axum::body::{Body, Bytes};

/// Withhold the second half until cancellation: EOF cannot satisfy a prefix test.
pub(super) fn held_response() -> Body {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        for _ in 0..super::TOTAL / 2 / (16 * 1024) {
            tokio::time::sleep(core::time::Duration::from_millis(10)).await;
            if tx
                .send(Ok::<_, std::io::Error>(Bytes::from(vec![7; 16 * 1024])))
                .await
                .is_err()
            {
                return;
            }
        }
        tx.closed().await;
    });
    Body::from_stream(tokio_stream::wrappers::ReceiverStream::new(rx))
}
