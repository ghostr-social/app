use super::HlsCacheMetadata;
use crate::segmented::prepare::{PreparedComplete, PreparedObject};
use crate::segmented::CachedHlsGeneration;

use std::sync::Arc;
use url::Url;

pub(super) struct StagedObject {
    request_url: String,
    final_url: Url,
    content_type: Option<String>,
    cache: HlsCacheMetadata,
    blocks: Vec<Arc<[u8]>>,
    bytes: u64,
    complete: bool,
    generation: Option<CachedHlsGeneration>,
}

pub(in crate::segmented) struct AssemblySeed {
    pub(in crate::segmented) request_url: String,
    pub(in crate::segmented) final_url: Url,
    pub(in crate::segmented) content_type: Option<String>,
    pub(in crate::segmented) cache: HlsCacheMetadata,
    pub(in crate::segmented) blocks: Vec<Arc<[u8]>>,
    pub(in crate::segmented) bytes: u64,
}

impl StagedObject {
    pub(super) fn partial(object: PreparedObject) -> Self {
        Self::new(object, false)
    }

    fn complete(object: PreparedObject) -> Self {
        Self::new(object, true)
    }

    fn new(object: PreparedObject, complete: bool) -> Self {
        let bytes = object.body.len() as u64;
        Self {
            request_url: object.request_url,
            final_url: object.final_url,
            content_type: object.content_type,
            cache: object.cache,
            blocks: vec![object.body],
            bytes,
            complete,
            generation: None,
        }
    }

    pub(super) fn complete_prepared(prepared: PreparedComplete) -> Self {
        let generation = prepared.generation;
        let mut staged = Self::complete(prepared.object);
        staged.generation = Some(generation);
        staged
    }

    pub(super) fn request_url(&self) -> &str {
        &self.request_url
    }

    pub(super) const fn len(&self) -> u64 {
        self.bytes
    }

    pub(super) fn is_assembled(&self) -> bool {
        self.complete && self.blocks.len() == 1
    }

    pub(super) fn continuation_len(&self, block: &PreparedObject, offset: u64) -> Option<u64> {
        self.matches(block, offset)
            .then(|| self.bytes.checked_add(block.body.len() as u64))?
    }

    pub(super) fn push(&mut self, block: PreparedObject, offset: u64) -> Option<()> {
        let bytes = self.continuation_len(&block, offset)?;
        self.cache = self.cache.combined_with(&block.cache);
        self.blocks.push(block.body);
        self.bytes = bytes;
        self.complete = false;
        Some(())
    }

    pub(super) fn into_prepared(mut self) -> Option<PreparedComplete> {
        let body = self.is_assembled().then(|| self.blocks.pop())??;
        Some(PreparedComplete {
            object: self.prepared(body, self.cache.clone()),
            generation: self.generation?,
        })
    }

    pub(super) fn matches_identity(&self, object: &PreparedObject, offset: u64) -> bool {
        self.bytes == offset
            && self.request_url == object.request_url
            && self.final_url == object.final_url
            && self.cache.validator == object.cache.validator
    }

    pub(super) fn assembly_seed(&self) -> AssemblySeed {
        AssemblySeed {
            request_url: self.request_url.clone(),
            final_url: self.final_url.clone(),
            content_type: self.content_type.clone(),
            cache: self.cache.clone(),
            blocks: self.blocks.clone(),
            bytes: self.bytes,
        }
    }

    fn matches(&self, block: &PreparedObject, offset: u64) -> bool {
        self.matches_identity(block, offset)
    }

    fn prepared(&self, body: Arc<[u8]>, cache: HlsCacheMetadata) -> PreparedObject {
        PreparedObject {
            request_url: self.request_url.clone(),
            final_url: self.final_url.clone(),
            body,
            content_type: self.content_type.clone(),
            cache,
        }
    }
}

#[cfg(test)]
#[path = "staged_object_axiom_test.rs"]
mod axiom_test_support;
