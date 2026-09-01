use axum::body::Bytes;
use core::convert::Infallible;
use core::ops::Range;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct ActiveRequest {
    pub path: String,
    pub range: Range<u64>,
    pub(super) body: mpsc::Sender<Result<Bytes, Infallible>>,
    pub(super) bytes: Arc<Vec<u8>>,
}

impl ActiveRequest {
    pub async fn send_bytes(&self, length: usize) -> bool {
        let start = self.range.start as usize;
        let end = start.saturating_add(length).min(self.range.end as usize);
        self.body
            .send(Ok(Bytes::copy_from_slice(&self.bytes[start..end])))
            .await
            .is_ok()
    }

    pub fn is_open(&self) -> bool {
        !self.body.is_closed()
    }
}
