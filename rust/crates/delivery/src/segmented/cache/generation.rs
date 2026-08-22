use ghostr_engine::evidence::EvidenceValidator;
use reqwest::header::HeaderMap;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Instant;
use url::Url;

const DOMAIN: &[u8] = b"ghostr:hls-cache-generation:v1";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CachedHlsGeneration([u8; 32]);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::segmented) struct HlsCacheMetadata {
    pub(super) validator: Option<EvidenceValidator>,
    pub(super) fresh_until: Option<Instant>,
}

#[derive(Clone)]
pub struct CachedHlsObject {
    pub body: Arc<[u8]>,
    pub final_url: Url,
    pub content_type: Option<String>,
    validator: Option<EvidenceValidator>,
    fresh_until: Option<Instant>,
    generation: CachedHlsGeneration,
}

impl CachedHlsObject {
    pub fn new(body: Arc<[u8]>, final_url: Url, content_type: Option<String>) -> Self {
        Self::with_metadata(body, final_url, content_type, HlsCacheMetadata::default())
    }

    pub(in crate::segmented) fn with_metadata(
        body: Arc<[u8]>,
        final_url: Url,
        content_type: Option<String>,
        metadata: HlsCacheMetadata,
    ) -> Self {
        let generation =
            CachedHlsGeneration::for_object(&final_url, body.as_ref(), metadata.validator.as_ref());
        Self {
            body,
            final_url,
            content_type,
            validator: metadata.validator,
            fresh_until: metadata.fresh_until,
            generation,
        }
    }

    pub fn generation(&self) -> CachedHlsGeneration {
        self.generation
    }

    pub fn validator(&self) -> Option<&EvidenceValidator> {
        self.validator.as_ref()
    }

    pub fn is_reusable(&self) -> bool {
        self.validator.is_some() && self.fresh_until.is_some_and(|until| Instant::now() < until)
    }
}

impl CachedHlsGeneration {
    fn for_object(final_url: &Url, body: &[u8], validator: Option<&EvidenceValidator>) -> Self {
        let mut digest = Sha256::new();
        digest.update(DOMAIN);
        digest.update((final_url.as_str().len() as u64).to_be_bytes());
        digest.update(final_url.as_str().as_bytes());
        digest.update((body.len() as u64).to_be_bytes());
        digest.update(body);
        hash_validator(&mut digest, validator);
        Self(digest.finalize().into())
    }

    pub fn for_response(final_url: &Url, body: &[u8], headers: &HeaderMap) -> Self {
        Self::for_object(
            final_url,
            body,
            super::freshness::response_validator(headers).as_ref(),
        )
    }
}

fn hash_validator(digest: &mut Sha256, validator: Option<&EvidenceValidator>) {
    match validator {
        Some(EvidenceValidator::StrongEtag(value)) => hash_field(digest, 1, value),
        Some(EvidenceValidator::LastModified(value)) => hash_field(digest, 2, value),
        None => digest.update([0]),
    }
}

fn hash_field(digest: &mut Sha256, kind: u8, value: &str) {
    digest.update([kind]);
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value.as_bytes());
}
