use super::{CatalogEntry, MediaLayout};
use crate::media_timeline::StartupFootprint;
use crate::ByteRange;

pub(super) struct Inputs<'a> {
    pub(super) entry: &'a CatalogEntry,
    pub(super) layout: MediaLayout,
    pub(super) total: Option<u64>,
    pub(super) duration_ms: u64,
    pub(super) present: &'a [ByteRange],
}

pub(super) fn footprint(inputs: Inputs<'_>) -> Option<StartupFootprint> {
    let total = inputs.total?;
    if covers_whole(total, inputs.present) || inputs.layout == MediaLayout::RequiresCompleteFile {
        return StartupFootprint::whole(total, inputs.duration_ms);
    }
    (inputs.layout == MediaLayout::Streamable)
        .then(|| inputs.entry.timeline()?.startup_footprint())
        .flatten()
}

fn covers_whole(total: u64, present: &[ByteRange]) -> bool {
    crate::media_timeline::normalize(present.to_vec())
        .first()
        .is_some_and(|range| range.start == 0 && range.end >= total)
}
