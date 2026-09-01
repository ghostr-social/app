use axum::body::Bytes;
use core::convert::Infallible;
use core::ops::Range;
use tokio::sync::mpsc;

pub struct ActiveRequest {
    pub path: String,
    pub range: Range<u64>,
    pub(super) body: mpsc::Sender<Result<Bytes, Infallible>>,
}

impl ActiveRequest {
    pub async fn send_byte(&self) -> bool {
        self.body.send(Ok(Bytes::from_static(&[7]))).await.is_ok()
    }

    pub async fn send_bytes(&self, length: usize) -> bool {
        self.body
            .send(Ok(Bytes::from(vec![7; length])))
            .await
            .is_ok()
    }

    pub fn is_open(&self) -> bool {
        !self.body.is_closed()
    }
}
