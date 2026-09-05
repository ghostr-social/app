use reqwest::{header::HeaderMap, Response, StatusCode, Url};

pub(super) struct ResponseHead {
    pub(super) status: StatusCode,
    pub(super) headers: HeaderMap,
    pub(super) url: Url,
    pub(super) content_length: Option<u64>,
    pub(super) status_error: Option<reqwest::Error>,
}

impl ResponseHead {
    pub(super) fn capture(response: &Response) -> Self {
        Self {
            status: response.status(),
            headers: response.headers().clone(),
            url: response.url().clone(),
            content_length: response.content_length(),
            status_error: response.error_for_status_ref().err(),
        }
    }
}
