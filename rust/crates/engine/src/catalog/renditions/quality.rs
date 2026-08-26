use super::RenditionState;
use crate::catalog::Catalog;
use crate::representation::RepresentationId;
use crate::PostId;

#[cfg(test)]
mod api_test;

const QUALITY_SCALE_MICROS: u64 = 1_000_000;

/// Exact active rendition quality relative to a complete advertised ladder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenditionQualityEvidence {
    active_bitrate_bps: u64,
    ceiling_bitrate_bps: u64,
}

impl RenditionQualityEvidence {
    pub(crate) fn normalized_micros(self) -> u64 {
        let scaled = u128::from(self.active_bitrate_bps) * u128::from(QUALITY_SCALE_MICROS);
        (scaled / u128::from(self.ceiling_bitrate_bps)) as u64
    }
}

impl Catalog {
    pub fn rendition_quality(&self, post: &PostId) -> Option<RenditionQualityEvidence> {
        let entry = self.lookup(post)?;
        entry.renditions.quality(entry.binding.representation())
    }
}

impl RenditionState {
    fn quality(&self, active: &RepresentationId) -> Option<RenditionQualityEvidence> {
        if self.variants.is_empty()
            || self
                .variants
                .iter()
                .any(|variant| variant.bitrate_bits_per_second().is_none())
        {
            return None;
        }
        let ladder = self
            .variants
            .iter()
            .filter_map(|variant| variant.quality())
            .collect();
        crate::rendition::RenditionSet::try_new(ladder).ok()?;
        let active_bitrate_bps = self.active_bitrate(active)?;
        let ceiling_bitrate_bps = self
            .variants
            .iter()
            .filter_map(|variant| variant.bitrate_bits_per_second())
            .max()?;
        Some(RenditionQualityEvidence {
            active_bitrate_bps,
            ceiling_bitrate_bps,
        })
    }
}
