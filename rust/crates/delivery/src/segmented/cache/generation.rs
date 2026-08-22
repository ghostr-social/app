use sha2::{Digest, Sha256};
use std::sync::Arc;
use url::Url;

const DOMAIN: &[u8] = b"ghostr:hls-cache-generation:v1";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CachedHlsGeneration([u8; 32]);

#[derive(Clone)]
pub struct CachedHlsObject {
    pub body: Arc<[u8]>,
    pub final_url: Url,
    pub content_type: Option<String>,
    generation: CachedHlsGeneration,
}

impl CachedHlsObject {
    pub fn new(body: Arc<[u8]>, final_url: Url, content_type: Option<String>) -> Self {
        let generation = CachedHlsGeneration::for_object(&final_url, &body);
        Self {
            body,
            final_url,
            content_type,
            generation,
        }
    }

    pub fn generation(&self) -> CachedHlsGeneration {
        self.generation
    }
}

impl CachedHlsGeneration {
    pub(super) fn for_object(final_url: &Url, body: &Arc<[u8]>) -> Self {
        let mut digest = Sha256::new();
        digest.update(DOMAIN);
        digest.update((final_url.as_str().len() as u64).to_be_bytes());
        digest.update(final_url.as_str().as_bytes());
        digest.update((body.len() as u64).to_be_bytes());
        digest.update(body.as_ref());
        Self(digest.finalize().into())
    }
}
