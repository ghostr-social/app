use super::{Catalog, CatalogEntry};
use crate::{EngineParams, PostId};

impl Catalog {
    /// Active rendition bitrate, then measured size/duration, then fallback.
    pub fn estimated_bitrate(&self, post: &PostId, params: &EngineParams) -> u64 {
        self.lookup(post)
            .and_then(|entry| active_or_measured(entry, entry.total_bytes()))
            .unwrap_or(params.assumed_bitrate_bps)
    }

    pub(crate) fn estimated_bitrate_for_total(
        &self,
        post: &PostId,
        total: Option<u64>,
        params: &EngineParams,
    ) -> u64 {
        self.lookup(post)
            .and_then(|entry| active_or_measured(entry, total))
            .unwrap_or(params.assumed_bitrate_bps)
    }
}

fn active_or_measured(entry: &CatalogEntry, total: Option<u64>) -> Option<u64> {
    entry
        .renditions
        .active_bitrate(entry.binding.representation())
        .or_else(|| measured_bitrate(entry, total))
}

fn measured_bitrate(entry: &CatalogEntry, bytes: Option<u64>) -> Option<u64> {
    let bytes = bytes?;
    let duration_ms = entry.meta.duration_ms.filter(|ms| *ms > 0)?;
    Some(bytes.saturating_mul(8).saturating_mul(1_000) / duration_ms)
}
