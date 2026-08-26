use super::RenditionQualityEvidence;

impl RenditionQualityEvidence {
    pub(crate) const fn active_bitrate_bps(self) -> u64 {
        self.active_bitrate_bps
    }

    pub(crate) const fn ceiling_bitrate_bps(self) -> u64 {
        self.ceiling_bitrate_bps
    }
}
