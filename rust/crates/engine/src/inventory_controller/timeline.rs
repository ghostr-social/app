use crate::catalog::CatalogEntry;
use crate::media_timeline::PlaybackWindow;
use crate::ByteRange;

pub(super) fn is_startable(
    entry: &CatalogEntry,
    have: &[ByteRange],
    head_seconds: u64,
) -> Option<bool> {
    let timeline = entry.timeline()?;
    let end = head_seconds.saturating_mul(1_000).max(1);
    let window = PlaybackWindow::try_new(0, end).expect("positive startup horizon");
    Some(
        timeline
            .required_ranges(window)
            .into_iter()
            .all(|range| covers(range, have)),
    )
}

fn covers(range: ByteRange, have: &[ByteRange]) -> bool {
    let mut cursor = range.start;
    while cursor < range.end {
        let Some(reach) = have
            .iter()
            .filter(|span| span.start <= cursor && span.end > cursor)
            .map(|span| span.end)
            .max()
        else {
            return false;
        };
        cursor = reach.min(range.end);
    }
    true
}
