//! Stable media identity plus a runtime generation for same-post refreshes.

use crate::{DeliveryKind, PostId, VideoMeta};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RepresentationId(String);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RepresentationGeneration(u64);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SourceId(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepresentationBinding {
    post: PostId,
    representation: RepresentationId,
    generation: RepresentationGeneration,
    sources: Vec<SourceId>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TransferIdentity {
    post: PostId,
    representation: RepresentationId,
    generation: RepresentationGeneration,
    source: SourceId,
}

impl RepresentationId {
    pub(crate) fn from_meta(meta: &VideoMeta) -> Self {
        let mut digest = Sha256::new();
        digest.update([delivery_tag(meta.delivery)]);
        match &meta.sha256 {
            Some(advertised) => field(&mut digest, advertised.as_bytes()),
            None => hash_unverified(&mut digest, meta),
        }
        Self(format!("{:x}", digest.finalize()))
    }

    pub fn fingerprint(&self) -> &str {
        &self.0
    }
}

impl RepresentationGeneration {
    pub(crate) fn first() -> Self {
        Self(1)
    }

    pub(crate) fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("representation generation exhausted"),
        )
    }
}

impl SourceId {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RepresentationBinding {
    pub(crate) fn new(
        post: PostId,
        meta: &VideoMeta,
        generation: RepresentationGeneration,
    ) -> Self {
        Self {
            post,
            representation: RepresentationId::from_meta(meta),
            generation,
            sources: meta.urls.iter().cloned().map(SourceId::new).collect(),
        }
    }

    pub fn post(&self) -> &PostId {
        &self.post
    }

    pub fn representation(&self) -> &RepresentationId {
        &self.representation
    }

    pub fn transfer(&self, url: &str) -> Option<TransferIdentity> {
        let source = self.sources.iter().find(|source| source.as_str() == url)?;
        Some(TransferIdentity {
            post: self.post.clone(),
            representation: self.representation.clone(),
            generation: self.generation,
            source: source.clone(),
        })
    }
}

impl TransferIdentity {
    pub fn post(&self) -> &PostId {
        &self.post
    }

    pub fn source(&self) -> &SourceId {
        &self.source
    }
}

fn hash_unverified(digest: &mut Sha256, meta: &VideoMeta) {
    let mut urls = meta.urls.clone();
    urls.sort();
    urls.dedup();
    for url in urls {
        field(digest, url.as_bytes());
    }
    optional_number(digest, meta.size_bytes);
    optional_number(digest, meta.duration_ms);
}

fn field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn optional_number(digest: &mut Sha256, value: Option<u64>) {
    digest.update([u8::from(value.is_some())]);
    digest.update(value.unwrap_or_default().to_be_bytes());
}

fn delivery_tag(delivery: DeliveryKind) -> u8 {
    match delivery {
        DeliveryKind::Progressive => 0,
        DeliveryKind::Hls => 1,
    }
}
