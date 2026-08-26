use super::outcome::{TimelineIncomplete, TimelineRejection, TimelineTerminal};
use ghostr_engine::media_timeline::{
    parse_mp4_segments_with_control, MediaSegment, TimelineError, TimelineParseControl,
};

pub(crate) struct TimelineInput {
    total: u64,
    segments: Vec<(u64, Vec<u8>)>,
}

impl TimelineInput {
    pub(super) fn new(total: u64, segments: Vec<(u64, Vec<u8>)>) -> Self {
        Self { total, segments }
    }
}

pub(crate) trait TimelineParser: Send + Sync {
    fn parse(&self, input: TimelineInput, control: &dyn TimelineParseControl) -> TimelineParse;
}

pub(crate) enum TimelineParse {
    Cancelled,
    Completed(TimelineTerminal),
}

pub(super) struct ProductionTimelineParser;

impl TimelineParser for ProductionTimelineParser {
    fn parse(&self, input: TimelineInput, control: &dyn TimelineParseControl) -> TimelineParse {
        let segments: Vec<_> = input
            .segments
            .iter()
            .map(|(start, bytes)| MediaSegment::new(*start, bytes))
            .collect();
        match parse_mp4_segments_with_control(&segments, control) {
            Ok(timeline) if timeline.fits_within(input.total) => {
                TimelineParse::Completed(TimelineTerminal::Ready(Box::new(timeline)))
            }
            Ok(_) => {
                TimelineParse::Completed(TimelineTerminal::Rejected(TimelineRejection::OutOfBounds))
            }
            Err(error) => terminal(error),
        }
    }
}

fn terminal(error: TimelineError) -> TimelineParse {
    let terminal = match error {
        TimelineError::Cancelled => return TimelineParse::Cancelled,
        TimelineError::Unavailable => TimelineTerminal::Incomplete(TimelineIncomplete::Unavailable),
        TimelineError::Truncated => TimelineTerminal::Incomplete(TimelineIncomplete::Truncated),
        TimelineError::Malformed => TimelineTerminal::Rejected(TimelineRejection::Malformed),
        TimelineError::ResourceLimit => {
            TimelineTerminal::Rejected(TimelineRejection::ResourceLimit)
        }
        TimelineError::Unsupported => TimelineTerminal::Rejected(TimelineRejection::Unsupported),
    };
    TimelineParse::Completed(terminal)
}
