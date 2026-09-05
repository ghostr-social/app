use super::gate::RequestLease;
use bytes::Bytes;
use core::time::Duration;
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode, Url};

pub struct MediaResponse {
    inner: Option<Response>,
    head: head::ResponseHead,
    lease: Option<RequestLease>,
    failed: bool,
    redirect_admission_wait_nanos: u64,
    selection: selection::ResponseSelection,
}

mod head;
pub(super) mod selection;

impl MediaResponse {
    pub(super) fn new(
        inner: Response,
        lease: RequestLease,
        redirect_admission_wait: Duration,
        selection: selection::ResponseSelection,
    ) -> Self {
        Self {
            head: head::ResponseHead::capture(&inner),
            inner: Some(inner),
            lease: Some(lease),
            failed: false,
            redirect_admission_wait_nanos: u64::try_from(redirect_admission_wait.as_nanos())
                .unwrap_or(u64::MAX),
            selection,
        }
    }

    pub fn status(&self) -> StatusCode {
        self.head.status
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.head.headers
    }

    pub fn url(&self) -> &Url {
        &self.head.url
    }

    pub fn content_length(&self) -> Option<u64> {
        self.head.content_length
    }

    pub fn request_selection(&self) -> Option<ghostr_engine::representation::RequestSelection> {
        self.selection.identity()
    }

    pub fn retention(&self) -> crate::media_retention::MediaRetention {
        self.selection.retention(self.headers(), self.url())
    }

    pub fn redirect_admission_wait(&self) -> Duration {
        Duration::from_nanos(self.redirect_admission_wait_nanos)
    }

    pub fn origin_elapsed(&self, elapsed: Duration) -> Duration {
        elapsed.saturating_sub(self.redirect_admission_wait())
    }

    /// # Errors
    ///
    /// Returns an error when the next response chunk cannot be read.
    pub async fn chunk(&mut self) -> anyhow::Result<Option<Bytes>> {
        anyhow::ensure!(!self.failed, "media response body is closed after failure");
        let result = self.read_chunk().await;
        if result.is_err() || matches!(result, Ok(None)) {
            self.failed = result.is_err();
            self.inner = None;
            self.lease = None;
        }
        result
    }

    async fn read_chunk(&mut self) -> anyhow::Result<Option<Bytes>> {
        let Some(response) = &mut self.inner else {
            return Ok(None);
        };
        let chunk = response.chunk().await?;
        let Some(lease) = &mut self.lease else {
            return Ok(None);
        };
        if let Some(bytes) = &chunk {
            lease.record_response_bytes(bytes.len() as u64);
            lease.received_body(bytes.len() as u64)?;
        } else {
            lease.complete_body()?;
        }
        Ok(chunk)
    }

    /// # Errors
    ///
    /// Returns an error when the response status is not successful.
    pub fn error_for_status(mut self) -> reqwest::Result<Self> {
        if let Some(error) = self.head.status_error.take() {
            return Err(error);
        }
        Ok(self)
    }
}
