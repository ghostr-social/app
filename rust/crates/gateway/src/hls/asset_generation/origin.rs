use crate::hls::asset_response::AssetResponseEnvelope;
use ghostr_net::media_request_executor::MediaResponse;
use ghostr_net::strong_etag::{single_strong_etag, StrongEtag};
use reqwest::header::HeaderValue;
use reqwest::Url;
use sha2::{Digest, Sha256};

const URL_DOMAIN: &[u8] = b"ghostr:hls-final-url:v1";
const MAX_ETAG_BYTES: usize = 1_024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::hls) struct OriginGeneration {
    final_url: [u8; 32],
    strong_etag: StrongEtag,
    total: u64,
}

impl OriginGeneration {
    pub fn if_range(&self) -> HeaderValue {
        self.strong_etag.as_header_value().clone()
    }

    pub(super) fn observed(response: &MediaResponse, total: Option<u64>) -> Option<Self> {
        let strong_etag = single_strong_etag(response.headers()).ok().flatten()?;
        if strong_etag.as_bytes().len() > MAX_ETAG_BYTES {
            return None;
        }
        Some(Self::new(
            response.url(),
            strong_etag,
            total.filter(|value| *value > 0)?,
        ))
    }

    pub(super) fn new(final_url: &Url, strong_etag: StrongEtag, total: u64) -> Self {
        Self {
            final_url: fingerprint(final_url),
            strong_etag,
            total,
        }
    }

    pub(super) fn matches(
        &self,
        envelope: AssetResponseEnvelope,
        response: &MediaResponse,
    ) -> bool {
        if self.final_url != fingerprint(response.url()) {
            return false;
        }
        match envelope {
            AssetResponseEnvelope::Partial { total, .. } => {
                total == Some(self.total) && self.etag_matches(response)
            }
            AssetResponseEnvelope::Unsatisfiable { total } => total == Some(self.total),
            AssetResponseEnvelope::Full { .. } => false,
        }
    }

    fn etag_matches(&self, response: &MediaResponse) -> bool {
        matches!(
            single_strong_etag(response.headers()),
            Ok(Some(found)) if found == self.strong_etag
        )
    }
}

fn fingerprint(url: &Url) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(URL_DOMAIN);
    digest.update(url.as_str().as_bytes());
    digest.finalize().into()
}
