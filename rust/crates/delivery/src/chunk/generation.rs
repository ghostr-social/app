use anyhow::{bail, Context, Result};
use ghostr_engine::representation::SourceGeneration;
use reqwest::header::{CONTENT_ENCODING, ETAG};
use reqwest::Response;

/// Response identity inspected before any sparse bytes are exposed.
pub struct OriginGeneration {
    final_url: String,
    strong_etag: Option<String>,
    total_bytes: Option<u64>,
}

impl OriginGeneration {
    pub(crate) fn from_response(response: &Response, total_bytes: Option<u64>) -> Result<Self> {
        require_identity_encoding(response)?;
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
}

fn require_identity_encoding(response: &Response) -> Result<()> {
    let Some(value) = response.headers().get(CONTENT_ENCODING) else {
        return Ok(());
    };
    if value
        .to_str()
        .ok()
        .is_some_and(|value| value.eq_ignore_ascii_case("identity"))
    {
        return Ok(());
    }
    bail!("encoded response cannot be assembled into sparse bytes")
}

fn is_strong_etag(value: &str) -> bool {
    value.starts_with('"')
        && value.ends_with('"')
        && value.len() >= 2
        && !value.bytes().any(|byte| byte.is_ascii_control())
}
