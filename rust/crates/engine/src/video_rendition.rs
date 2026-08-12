//! One independently encoded progressive representation and its mirrors.

use crate::rendition::{Rendition, RenditionId};
use crate::representation::RepresentationId;
use crate::{DeliveryKind, VideoMeta};
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoRenditionError {
    EmptySources,
    NotProgressive,
    ZeroBitrate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoRendition {
    meta: VideoMeta,
    bitrate_bps: Option<NonZeroU64>,
}

impl VideoRendition {
    pub fn try_new(meta: VideoMeta, bitrate_bps: Option<u64>) -> Result<Self, VideoRenditionError> {
        if meta.urls.is_empty() {
            return Err(VideoRenditionError::EmptySources);
        }
        if meta.delivery != DeliveryKind::Progressive {
            return Err(VideoRenditionError::NotProgressive);
        }
        let bitrate_bps = bitrate_bps
            .map(|value| NonZeroU64::new(value).ok_or(VideoRenditionError::ZeroBitrate))
            .transpose()?;
        Ok(Self { meta, bitrate_bps })
    }

    pub fn meta(&self) -> &VideoMeta {
        &self.meta
    }

    pub fn bitrate_bits_per_second(&self) -> Option<u64> {
        self.bitrate_bps.map(NonZeroU64::get)
    }

    pub(crate) fn identity(&self) -> RepresentationId {
        RepresentationId::from_meta(&self.meta)
    }

    pub(crate) fn quality(&self) -> Option<Rendition> {
        let bitrate = self.bitrate_bits_per_second()?;
        Rendition::try_new(self.identity().fingerprint(), bitrate).ok()
    }

    pub(crate) fn quality_id(&self) -> RenditionId {
        RenditionId::try_new(self.identity().fingerprint()).expect("representation id is non-empty")
    }
}
