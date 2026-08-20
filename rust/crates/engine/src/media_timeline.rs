//! Pure MP4/CMAF timing evidence. Container bytes enter here as bounded,
//! absolute segments and leave as validated time-to-byte mappings.

use crate::ByteRange;

mod boxes;
mod classic;
mod control;
mod limits;
mod ranges;
mod segments;
mod selection;
mod sidx;
mod startup;

pub use control::TimelineParseControl;
pub use ranges::normalize;
pub use startup::{StartupFootprint, StartupProvenance};

/// A contiguous piece of the remote representation already held locally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MediaSegment<'a> {
    pub(crate) start: u64,
    pub(crate) bytes: &'a [u8],
}

impl<'a> MediaSegment<'a> {
    pub const fn new(start: u64, bytes: &'a [u8]) -> Self {
        Self { start, bytes }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineError {
    Cancelled,
    Unavailable,
    Truncated,
    Malformed,
    ResourceLimit,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaTimeline {
    pub(crate) metadata: Vec<ByteRange>,
    pub(crate) file_types: Vec<ByteRange>,
    pub(crate) movie: Option<ByteRange>,
    pub(crate) movie_top_level: bool,
    pub(crate) media_data: Vec<boxes::MediaData>,
    pub(crate) classic_video: bool,
    pub(crate) media: Vec<TimedRange>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayableExtent {
    pub(crate) bytes: ByteRange,
    pub(crate) playable_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TimedRange {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
    pub(crate) bytes: ByteRange,
}

impl MediaTimeline {
    pub fn front_moov(&self) -> bool {
        let Some(movie) = self.movie else {
            return false;
        };
        let first_media = self
            .media
            .iter()
            .map(|range| range.bytes.start)
            .min()
            .unwrap_or(u64::MAX);
        self.movie_top_level && movie.end <= first_media
    }

    pub fn duration_ms(&self) -> Option<u64> {
        let start = self.media.iter().map(|range| range.start_ms).min()?;
        let end = self.media.iter().map(|range| range.end_ms).max()?;
        (end > start).then_some(end - start)
    }

    pub fn fits_within(&self, total_bytes: u64) -> bool {
        self.metadata
            .iter()
            .chain(self.media_data.iter().map(|data| &data.payload))
            .chain(self.media.iter().map(|range| &range.bytes))
            .all(|range| range.end <= total_bytes)
    }

    pub(crate) fn playable_extents(&self) -> Vec<PlayableExtent> {
        self.media
            .iter()
            .map(|range| PlayableExtent {
                bytes: range.bytes,
                playable_ms: range.end_ms.saturating_sub(range.start_ms).max(1),
            })
            .collect()
    }

    pub fn startup_footprint(&self) -> Option<StartupFootprint> {
        StartupFootprint::from_timeline(self)
    }
}

/// Parses complete metadata boxes found in the supplied absolute segments.
/// Segments may be disjoint or adjacent, but their non-empty ranges must not
/// overlap. Input order does not affect the result.
pub fn parse_mp4_segments(segments: &[MediaSegment<'_>]) -> Result<MediaTimeline, TimelineError> {
    parse_mp4_segments_with_control(segments, &control::NeverCancelled)
}

pub fn parse_mp4_segments_with_control(
    segments: &[MediaSegment<'_>],
    control: &dyn TimelineParseControl,
) -> Result<MediaTimeline, TimelineError> {
    let mut budget = limits::ParserBudget::new(segments, control)?;
    let segments = segments::canonical(segments, &mut budget)?;
    let scan = boxes::scan(&segments, &mut budget)?;
    let selected = selection::parse(&scan.atoms, scan.truncated, &mut budget)?;
    startup::assemble(
        startup::AssemblyInput {
            atoms: &scan.atoms,
            media_data: scan.media_data,
            media: selected.ranges,
            movie: selected.classic_movie,
            movie_top_level: selected.classic_movie_top_level,
            classic_video: selected.classic_video,
        },
        &mut budget,
    )
}
