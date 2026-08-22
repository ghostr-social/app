use crate::segmented::fetch::FetchedObject;
use anyhow::{bail, Result};
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_hls_manifest::hls_manifest::{inspect_hls_bootstrap, HlsBootstrap};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Pending {
    pub generation: u64,
    pub source_index: usize,
    pub stage: HlsBootstrapStage,
    pub url: String,
    pub after_init: Option<String>,
}

pub(super) enum Advance {
    Pending(Pending),
    Ready,
}

impl Pending {
    pub(super) fn root(generation: u64, source_index: usize, url: String) -> Self {
        Self {
            generation,
            source_index,
            stage: HlsBootstrapStage::RootManifest,
            url,
            after_init: None,
        }
    }

    pub(super) fn advance(&self, object: &FetchedObject) -> Result<Advance> {
        match self.stage {
            HlsBootstrapStage::RootManifest | HlsBootstrapStage::ChildPlaylist => {
                self.advance_manifest(object)
            }
            HlsBootstrapStage::Initialization => self.advance_init(),
            HlsBootstrapStage::FirstSegment => Ok(Advance::Ready),
        }
    }

    fn advance_manifest(&self, object: &FetchedObject) -> Result<Advance> {
        match inspect_hls_bootstrap(&object.body, &object.final_url)? {
            HlsBootstrap::Master { variant } if self.stage == HlsBootstrapStage::RootManifest => {
                Ok(Advance::Pending(Self {
                    stage: HlsBootstrapStage::ChildPlaylist,
                    url: variant.to_string(),
                    after_init: None,
                    ..self.clone()
                }))
            }
            HlsBootstrap::Master { .. } => bail!("nested HLS master exceeds depth limit"),
            HlsBootstrap::Media { init, segment } => Ok(Advance::Pending(Self {
                stage: init.as_ref().map_or(HlsBootstrapStage::FirstSegment, |_| {
                    HlsBootstrapStage::Initialization
                }),
                url: init.as_ref().unwrap_or(&segment).to_string(),
                after_init: init.map(|_| segment.to_string()),
                ..self.clone()
            })),
        }
    }

    fn advance_init(&self) -> Result<Advance> {
        let segment = self
            .after_init
            .clone()
            .ok_or_else(|| anyhow::anyhow!("HLS initialization has no first segment"))?;
        Ok(Advance::Pending(Self {
            stage: HlsBootstrapStage::FirstSegment,
            url: segment,
            after_init: None,
            ..self.clone()
        }))
    }
}
