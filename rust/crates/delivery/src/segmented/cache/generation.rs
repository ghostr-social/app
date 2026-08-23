use crate::segmented::prepare::PreparedComplete;
use ghostr_engine::evidence::EvidenceValidator;
use reqwest::header::HeaderMap;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Instant;
use url::Url;

const DOMAIN: &[u8] = b"ghostr:hls-cache-generation:v1";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CachedHlsGeneration([u8; 32]);

pub(in crate::segmented) struct CachedHlsGenerationHasher {
    digest: Sha256,
    validator: Option<EvidenceValidator>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::segmented) struct HlsCacheMetadata {
    pub(super) validator: Option<EvidenceValidator>,
    pub(super) fresh_until: Option<Instant>,
}

impl HlsCacheMetadata {
    pub(in crate::segmented) fn combined_with(&self, block: &Self) -> Self {
        let fresh_until = match (self.fresh_until, block.fresh_until) {
            (Some(left), Some(right)) => Some(left.min(right)),
            _ => None,
        };
        Self {
            validator: self.validator.clone(),
            fresh_until,
        }
    }
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

    pub(in crate::segmented) fn from_prepared(prepared: PreparedComplete) -> Self {
        let PreparedComplete { object, generation } = prepared;
        Self {
            body: object.body,
            final_url: object.final_url,
            content_type: object.content_type,
            validator: object.cache.validator,
            fresh_until: object.cache.fresh_until,
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
        let mut hasher = CachedHlsGenerationHasher::with_validator(
            final_url,
            body.len() as u64,
            validator.cloned(),
        );
        hasher.update(body);
        hasher.finish()
    }

    pub fn for_response(final_url: &Url, body: &[u8], headers: &HeaderMap) -> Self {
        Self::for_object(
            final_url,
            body,
            super::freshness::response_validator(headers).as_ref(),
        )
    }
}

impl CachedHlsGenerationHasher {
    pub(in crate::segmented) fn new(
        final_url: &Url,
        body_bytes: u64,
        metadata: &HlsCacheMetadata,
    ) -> Self {
        Self::with_validator(final_url, body_bytes, metadata.validator.clone())
    }

    fn with_validator(
        final_url: &Url,
        body_bytes: u64,
        validator: Option<EvidenceValidator>,
    ) -> Self {
        let mut digest = Sha256::new();
        digest.update(DOMAIN);
        digest.update((final_url.as_str().len() as u64).to_be_bytes());
        digest.update(final_url.as_str().as_bytes());
        digest.update(body_bytes.to_be_bytes());
        Self { digest, validator }
    }

    pub(in crate::segmented) fn update(&mut self, bytes: &[u8]) {
        self.digest.update(bytes);
    }

    pub(in crate::segmented) fn finish(mut self) -> CachedHlsGeneration {
        hash_validator(&mut self.digest, self.validator.as_ref());
        CachedHlsGeneration(self.digest.finalize().into())
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
