//! Stable media identity plus a runtime generation for same-post refreshes.
use crate::{PostId, VideoMeta};
use serde::{Deserialize, Serialize};
use std::fmt;

mod derived;
mod http_generation;
mod identity;
pub use http_generation::{
    HttpGenerationAuthority, HttpGenerationEpoch, HttpGenerationKey, HttpGenerationLease,
    HttpGenerationStamp, InvalidHttpGeneration,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RepresentationId(String);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RepresentationGeneration(u64);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SourceId(String);

/// One coherent sparse-byte generation returned by an origin.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SourceGeneration {
    final_url: String,
    strong_etag: String,
    total_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSourceGeneration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepresentationBinding {
    post: PostId,
    representation: RepresentationId,
    derived_from: Option<RepresentationId>,
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
        Self(identity::fingerprint(meta))
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
        let next = self
            .0
            .checked_add(1)
            .expect("representation generation exhausted");
        Self(next)
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

impl SourceGeneration {
    pub fn try_new(
        final_url: impl Into<String>,
        strong_etag: impl Into<String>,
        total_bytes: u64,
    ) -> Result<Self, InvalidSourceGeneration> {
        let generation = Self {
            final_url: final_url.into(),
            strong_etag: strong_etag.into(),
            total_bytes,
        };
        generation
            .is_valid()
            .then_some(generation)
            .ok_or(InvalidSourceGeneration)
    }

    pub fn final_url(&self) -> &str {
        &self.final_url
    }

    pub fn strong_etag(&self) -> &str {
        &self.strong_etag
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    fn is_valid(&self) -> bool {
        !self.final_url.is_empty()
            && self.total_bytes > 0
            && self.strong_etag.starts_with('"')
            && self.strong_etag.ends_with('"')
            && self.strong_etag.len() >= 2
            && !self.strong_etag.starts_with("W/")
            && !self.strong_etag.bytes().any(|byte| byte.is_ascii_control())
    }
}

impl fmt::Display for InvalidSourceGeneration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("sparse generation needs a final URL, strong ETag, and length")
    }
}

impl std::error::Error for InvalidSourceGeneration {}

impl RepresentationBinding {
    pub(crate) fn new(
        post: PostId,
        meta: &VideoMeta,
        generation: RepresentationGeneration,
    ) -> Self {
        Self {
            post,
            representation: RepresentationId::from_meta(meta),
            derived_from: None,
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

    pub fn matches_meta(&self, meta: &VideoMeta) -> bool {
        self.representation == RepresentationId::from_meta(meta)
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
