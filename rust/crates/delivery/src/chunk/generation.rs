use crate::chunk::downloader::HttpResponseEvidence;
use anyhow::{Context, Result};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::SourceGeneration;
use reqwest::header::{CONTENT_TYPE, ETAG, LAST_MODIFIED};
use reqwest::Response;

/// Response identity inspected before any sparse bytes are exposed.
pub struct OriginGeneration {
    final_url: String,
    strong_etag: Option<String>,
    total_bytes: Option<u64>,
}

impl HttpResponseEvidence {
    pub(crate) fn from_response(response: &Response) -> Self {
        let headers = response.headers();
        let etag = header(headers, &ETAG).and_then(EvidenceValidator::strong_etag);
        let modified =
            || header(headers, &LAST_MODIFIED).and_then(EvidenceValidator::last_modified);
        Self {
            final_url: response.url().to_string(),
            content_type: header(headers, &CONTENT_TYPE),
            validator: etag.or_else(modified),
        }
    }
}

impl OriginGeneration {
    pub(crate) fn from_response(response: &Response, total_bytes: Option<u64>) -> Result<Self> {
        let strong_etag = response
            .headers()
            .get(ETAG)
            .map(|value| value.to_str().context("origin ETag is not text"))
            .transpose()?
            .filter(|value| is_strong_etag(value))
            .map(str::to_owned);
        Ok(Self {
            final_url: response.url().to_string(),
            strong_etag,
            total_bytes,
        })
    }

    pub(crate) fn strict(&self) -> Result<SourceGeneration> {
        let etag = self
            .strong_etag
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("sparse response needs a strong ETag"))?;
        let total = self
            .total_bytes
            .ok_or_else(|| anyhow::anyhow!("sparse response needs a complete length"))?;
        SourceGeneration::try_new(&self.final_url, etag, total)
            .context("invalid sparse response generation")
    }

    pub(crate) fn is_resumable(&self) -> bool {
        self.strong_etag.is_some() && self.total_bytes.is_some()
    }

    pub(crate) fn resumable(&self) -> Option<SourceGeneration> {
        self.strict().ok()
    }
}

fn is_strong_etag(value: &str) -> bool {
    value.starts_with('"')
        && value.ends_with('"')
        && value.len() >= 2
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn header(
    headers: &reqwest::header::HeaderMap,
    name: &reqwest::header::HeaderName,
) -> Option<String> {
    headers.get(name)?.to_str().ok().map(str::to_owned)
}
