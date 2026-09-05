//! Pure MP4/CMAF timing evidence. Container bytes enter here as bounded,
//! absolute segments and leave as validated time-to-byte mappings.

use crate::ByteRange;

mod boxes;
mod classic;
pub mod compiled;
mod control;
mod limits;
mod ranges;
mod segments;
mod selection;
mod sidx;
mod startup;
mod timing;

pub use control::TimelineParseControl;
pub use ranges::normalize;
pub use startup::{StartupFootprint, StartupProvenance};

/// A contiguous piece of the remote representation already held locally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MediaSegment<'a> {
    start: u64,
    bytes: &'a [u8],
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

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MediaTimeline {
    inspected: Vec<ByteRange>,
    pub(super) metadata: Vec<ByteRange>,
    file_types: Vec<ByteRange>,
    movie: Option<ByteRange>,
    movie_top_level: bool,
    top_level_file_types: usize,
    top_level_movies: usize,
    fragmented_indexes: usize,
    media_data: Vec<boxes::MediaData>,
    classic_video: bool,
    #[serde(skip)]
    startup: Option<StartupFootprint>,
    pub(super) media: Vec<TimedRange>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayableExtent {
    pub(super) bytes: ByteRange,
    pub(super) playable_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TimedRange {
    pub(super) time: timing::PresentationTime,
    decode_start: Option<u64>,
    track: u16,
    sync_sample: Option<u32>,
    pub(super) bytes: ByteRange,
}

impl MediaTimeline {
    pub(super) fn front_moov(&self) -> bool {
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

    pub(super) fn duration_ms(&self) -> Option<u64> {
        let start = self.media.iter().map(|range| range.time.start_ms()).min()?;
        let end = self.media.iter().map(|range| range.time.end_ms()).max()?;
        (end > start).then_some(end - start)
    }

    pub fn fits_within(&self, total_bytes: u64) -> bool {
        self.metadata
            .iter()
            .chain(self.media_data.iter().map(|data| &data.payload))
            .chain(self.media.iter().map(|range| &range.bytes))
            .all(|range| range.end <= total_bytes)
    }

    pub(super) fn playable_extents(&self) -> Vec<PlayableExtent> {
        self.media
            .iter()
            .map(|range| PlayableExtent {
                bytes: range.bytes,
                playable_ms: range
                    .time
                    .end_ms()
                    .saturating_sub(range.time.start_ms())
                    .max(1),
            })
            .collect()
    }

    pub fn startup_footprint(&self) -> Option<StartupFootprint> {
        self.startup.clone()
    }

    pub fn fast_start_remuxable(&self, total: u64) -> bool {
        let Some(movie) = self.movie else {
            return false;
        };
        let [media] = self.media_data.as_slice() else {
            return false;
        };
        self.inspected_whole(total)
            && self.top_level_file_types == 1
            && self.top_level_movies == 1
            && self.fragmented_indexes == 0
            && self.startup_footprint().is_some()
            && media.payload.end <= movie.start
            && movie.end == total
    }

    fn inspected_whole(&self, total: u64) -> bool {
        normalize(self.inspected.clone())
            .first()
            .is_some_and(|range| range.start == 0 && range.end == total)
    }
}

/// Parses MP4 metadata while honoring an external cancellation control.
///
/// # Errors
///
/// Returns a timeline error for cancellation, invalid segment geometry, malformed MP4 data, or
/// exhausted parser limits.
pub fn parse_mp4_segments_with_control(
    segments: &[MediaSegment<'_>],
    control: &dyn TimelineParseControl,
) -> Result<MediaTimeline, TimelineError> {
    let mut budget = limits::ParserBudget::new(segments, control)?;
    let segments = segments::canonical(segments, &mut budget)?;
    let inspected = segments
        .iter()
        .map(|segment| ByteRange::new(segment.start, segment.start + segment.bytes.len() as u64))
        .collect();
    let scan = boxes::scan(&segments, &mut budget)?;
    let selected = selection::parse(&scan.atoms, scan.truncated, &mut budget)?;
    startup::assemble(
        startup::AssemblyInput {
            atoms: &scan.atoms,
            inspected,
            media_data: scan.media_data,
            fragmented_markers: scan.fragmented_markers,
            media: selected.ranges,
            movie: selected.classic_movie,
            movie_top_level: selected.classic_movie_top_level,
            classic_video: selected.classic_video,
        },
        &mut budget,
    )
}

#[cfg(any(test, feature = "test"))]
#[path = "media_timeline/test_support.rs"]
mod test_support;
#[cfg(any(test, feature = "test"))]
pub use test_support::parse_mp4_segments;
