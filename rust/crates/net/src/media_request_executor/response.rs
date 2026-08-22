use super::gate::RequestLease;
use bytes::Bytes;
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode, Url};
use std::time::Duration;

pub struct MediaResponse {
    inner: Response,
    lease: Option<RequestLease>,
    redirect_admission_wait_nanos: u64,
}

impl MediaResponse {
    pub(super) fn new(
        inner: Response,
        lease: RequestLease,
        redirect_admission_wait: Duration,
    ) -> Self {
        Self {
            inner,
            lease: Some(lease),
            redirect_admission_wait_nanos: u64::try_from(redirect_admission_wait.as_nanos())
                .unwrap_or(u64::MAX),
        }
    }

    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    pub fn url(&self) -> &Url {
        self.inner.url()
    }

    pub fn content_length(&self) -> Option<u64> {
        self.inner.content_length()
    }

    pub fn redirect_admission_wait(&self) -> Duration {
        Duration::from_nanos(self.redirect_admission_wait_nanos)
    }

    pub fn origin_elapsed(&self, elapsed: Duration) -> Duration {
        elapsed.saturating_sub(self.redirect_admission_wait())
    }

    pub async fn chunk(&mut self) -> reqwest::Result<Option<Bytes>> {
        let chunk = self.inner.chunk().await;
        if let Ok(Some(bytes)) = &chunk {
            if let Some(lease) = &self.lease {
                lease.record_response_bytes(bytes.len() as u64);
            }
        } else {
            self.lease = None;
        }
        chunk
    }

    pub fn error_for_status(self) -> reqwest::Result<Self> {
        self.inner.error_for_status_ref()?;
        Ok(self)
    }
}
