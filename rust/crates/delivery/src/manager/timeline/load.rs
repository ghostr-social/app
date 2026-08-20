use super::{
    metadata_ranges, TimelineIncomplete, TimelineInput, TimelineParse, TimelineParser,
    TimelineTerminal,
};
use ghostr_engine::media_timeline::MediaTimeline;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::PartialRangeStore;

pub(crate) async fn load_timeline(
    store: &PartialRangeStore,
    post: &PostId,
    total: u64,
    present: &[ByteRange],
) -> Option<MediaTimeline> {
    let ranges = metadata_ranges(total, present);
    let mut owned = Vec::with_capacity(ranges.len());
    for range in ranges {
        let bytes = store
            .read_range(post.as_str(), range.start..range.end)
            .await
            .ok()??;
        owned.push((range.start, bytes));
    }
    let parser = super::parser::ProductionTimelineParser;
    let control = std::sync::atomic::AtomicBool::new(false);
    match parser.parse(TimelineInput::new(total, owned), &control) {
        TimelineParse::Completed(TimelineTerminal::Ready(timeline)) => Some(timeline),
        TimelineParse::Completed(TimelineTerminal::Incomplete(
            TimelineIncomplete::Unavailable | TimelineIncomplete::Truncated,
        ))
        | TimelineParse::Completed(TimelineTerminal::Rejected(_))
        | TimelineParse::Cancelled => None,
    }
}
