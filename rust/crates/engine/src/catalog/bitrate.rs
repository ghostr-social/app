use super::{Catalog, CatalogEntry};
use crate::{EngineParams, PostId};

impl Catalog {
    /// Active rendition bitrate, then measured size/duration, then fallback.
    pub fn estimated_bitrate(&self, post: &PostId, params: &EngineParams) -> u64 {
        self.lookup(post)
            .and_then(active_or_measured)
            .unwrap_or(params.assumed_bitrate_bps)
    }
}

fn active_or_measured(entry: &CatalogEntry) -> Option<u64> {
    entry
        .renditions
        .active_bitrate(entry.binding.representation())
        .or_else(|| measured_bitrate(entry))
}

fn measured_bitrate(entry: &CatalogEntry) -> Option<u64> {
    let bytes = entry.total_bytes()?;
    let duration_ms = entry.meta.duration_ms.filter(|ms| *ms > 0)?;
    Some(bytes.saturating_mul(8).saturating_mul(1_000) / duration_ms)
}
