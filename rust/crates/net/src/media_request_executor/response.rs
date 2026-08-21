use super::gate::RequestLease;
use bytes::Bytes;
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode, Url};

pub struct MediaResponse {
    inner: Response,
    lease: Option<RequestLease>,
}

impl MediaResponse {
    pub(super) fn new(inner: Response, lease: RequestLease) -> Self {
        Self {
            inner,
            lease: Some(lease),
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

    pub async fn chunk(&mut self) -> reqwest::Result<Option<Bytes>> {
        let chunk = self.inner.chunk().await;
        if !matches!(chunk, Ok(Some(_))) {
            self.lease = None;
        }
        chunk
    }

    pub fn error_for_status(self) -> reqwest::Result<Self> {
        self.inner.error_for_status_ref()?;
        Ok(self)
    }
}
