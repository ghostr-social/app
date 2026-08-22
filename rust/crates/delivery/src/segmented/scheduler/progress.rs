use crate::segmented::fetch::ObjectContinuation;
use crate::segmented::prepare::PreparedObject;
use anyhow::{bail, Result};
use ghostr_engine::adaptive::{HlsBootstrapStage, HlsObjectCursor, HlsTransport};
use ghostr_hls_manifest::hls_manifest::{inspect_hls_bootstrap, HlsBootstrap};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Pending {
    pub generation: u64,
    pub attempt: u64,
    pub generation_restarts: u8,
    pub source_index: usize,
    pub root_source: String,
    pub stage: HlsBootstrapStage,
    pub url: String,
    pub after_init: Option<String>,
    pub continuation: Option<ObjectContinuation>,
}

pub(super) enum Advance {
    Pending(Box<Pending>),
    Ready,
}

impl Pending {
    pub(super) fn root(generation: u64, attempt: u64, source_index: usize, url: String) -> Self {
        Self {
            generation,
            attempt,
            generation_restarts: 0,
            source_index,
            root_source: url.clone(),
            stage: HlsBootstrapStage::RootManifest,
            url,
            after_init: None,
            continuation: None,
        }
    }

    pub(super) fn cursor(&self) -> HlsObjectCursor {
        self.continuation.as_ref().map_or_else(
            || HlsObjectCursor::new(self.attempt, 0, None, HlsTransport::Start),
            |continuation| {
                HlsObjectCursor::new(
                    self.attempt,
                    continuation.next_offset,
                    Some(continuation.total),
                    HlsTransport::ResumeRange,
                )
            },
        )
    }

    pub(super) fn with_attempt(mut self, attempt: u64) -> Self {
        self.attempt = attempt;
        self.generation_restarts = 0;
        self
    }

    pub(super) fn restart_object(mut self, attempt: u64) -> Self {
        self.attempt = attempt;
        self.generation_restarts = self.generation_restarts.saturating_add(1);
        self.continuation = None;
        self
    }

    pub(super) fn continued(&self, continuation: ObjectContinuation) -> Self {
        Self {
            continuation: Some(continuation),
            ..self.clone()
        }
    }

    pub(super) fn advance(&self, object: &PreparedObject) -> Result<Advance> {
        match self.stage {
            HlsBootstrapStage::RootManifest | HlsBootstrapStage::ChildPlaylist => {
                self.advance_manifest(object)
            }
            HlsBootstrapStage::Initialization => self.advance_init(),
            HlsBootstrapStage::FirstSegment => Ok(Advance::Ready),
        }
    }

    fn advance_manifest(&self, object: &PreparedObject) -> Result<Advance> {
        match inspect_hls_bootstrap(&object.body, &object.final_url)? {
            HlsBootstrap::Master { variant } if self.stage == HlsBootstrapStage::RootManifest => {
                Ok(Advance::Pending(Box::new(Self {
                    stage: HlsBootstrapStage::ChildPlaylist,
                    url: variant.to_string(),
                    after_init: None,
                    continuation: None,
                    ..self.clone()
                })))
            }
            HlsBootstrap::Master { .. } => bail!("nested HLS master exceeds depth limit"),
            HlsBootstrap::Media { init, segment } => Ok(Advance::Pending(Box::new(Self {
                stage: init.as_ref().map_or(HlsBootstrapStage::FirstSegment, |_| {
                    HlsBootstrapStage::Initialization
                }),
                url: init.as_ref().unwrap_or(&segment).to_string(),
                after_init: init.map(|_| segment.to_string()),
                continuation: None,
                ..self.clone()
            }))),
        }
    }

    fn advance_init(&self) -> Result<Advance> {
        let segment = self
            .after_init
            .clone()
            .ok_or_else(|| anyhow::anyhow!("HLS initialization has no first segment"))?;
        Ok(Advance::Pending(Box::new(Self {
            stage: HlsBootstrapStage::FirstSegment,
            url: segment,
            after_init: None,
            continuation: None,
            ..self.clone()
        })))
    }
}
