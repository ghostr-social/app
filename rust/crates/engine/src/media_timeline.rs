//! Pure MP4/CMAF timing evidence. Container bytes enter here as bounded,
//! absolute segments and leave as validated time-to-byte mappings.

use crate::ByteRange;

mod boxes;
mod classic;
mod sidx;

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

/// A validated half-open media-time interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlaybackWindow {
    start_ms: u64,
    end_ms: u64,
}

impl PlaybackWindow {
    pub fn try_new(start_ms: u64, end_ms: u64) -> Result<Self, PlaybackWindowError> {
        if start_ms >= end_ms {
            return Err(PlaybackWindowError);
        }
        Ok(Self { start_ms, end_ms })
    }

    pub(crate) const fn start_ms(self) -> u64 {
        self.start_ms
    }

    pub(crate) const fn end_ms(self) -> u64 {
        self.end_ms
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlaybackWindowError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineError {
    Unavailable,
    Truncated,
    Malformed,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaTimeline {
    metadata: Vec<ByteRange>,
    media: Vec<TimedRange>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TimedRange {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
    pub(crate) bytes: ByteRange,
}

impl MediaTimeline {
    pub fn duration_ms(&self) -> u64 {
        self.media
            .iter()
            .map(|range| range.end_ms)
            .max()
            .unwrap_or(0)
    }

    pub fn media_ranges(&self, window: PlaybackWindow) -> Vec<ByteRange> {
        normalize(
            self.media
                .iter()
                .filter(|range| overlaps(**range, window))
                .map(|range| range.bytes)
                .collect(),
        )
    }

    pub fn required_ranges(&self, window: PlaybackWindow) -> Vec<ByteRange> {
        let mut ranges = self.metadata.clone();
        ranges.extend(self.media_ranges(window));
        normalize(ranges)
    }

    pub fn fits_within(&self, total_bytes: u64) -> bool {
        self.metadata
            .iter()
            .chain(self.media.iter().map(|range| &range.bytes))
            .all(|range| range.end <= total_bytes)
    }

    pub(crate) fn from_parts(metadata: Vec<ByteRange>, media: Vec<TimedRange>) -> Self {
        Self {
            metadata: normalize(metadata),
            media,
        }
    }
}

/// Parses complete metadata boxes found in any of the supplied absolute
/// segments. Segments may be disjoint, which is essential for tail `moov`.
pub fn parse_mp4_segments(segments: &[MediaSegment<'_>]) -> Result<MediaTimeline, TimelineError> {
    let scan = boxes::scan(segments)?;
    let parsed = parse_atoms(&scan.atoms)?;
    let media = select_media(parsed, scan.truncated)?;
    Ok(MediaTimeline::from_parts(
        scan_metadata(&scan.atoms)?,
        media,
    ))
}

struct ParsedMedia {
    classic: Vec<TimedRange>,
    fragmented: Vec<TimedRange>,
}

fn parse_atoms(atoms: &[boxes::Atom<'_>]) -> Result<ParsedMedia, TimelineError> {
    let mut classic = Vec::new();
    let mut fragmented = Vec::new();
    for atom in atoms {
        match &atom.kind {
            b"moov" => classic.extend(classic::parse(atom)?),
            _ => fragmented.extend(sidx::parse(atom)?),
        }
    }
    Ok(ParsedMedia {
        classic,
        fragmented,
    })
}

fn scan_metadata(atoms: &[boxes::Atom<'_>]) -> Result<Vec<ByteRange>, TimelineError> {
    let mut metadata = Vec::new();
    for atom in atoms {
        metadata.push(atom.range()?);
    }
    Ok(metadata)
}

fn select_media(parsed: ParsedMedia, truncated: bool) -> Result<Vec<TimedRange>, TimelineError> {
    let media = match parsed.classic.is_empty() {
        false => parsed.classic,
        true => parsed.fragmented,
    };
    if media.is_empty() {
        return Err(if truncated {
            TimelineError::Truncated
        } else {
            TimelineError::Unavailable
        });
    }
    Ok(media)
}

fn overlaps(range: TimedRange, window: PlaybackWindow) -> bool {
    range.start_ms < window.end_ms() && range.end_ms > window.start_ms()
}

pub(crate) fn normalize(mut ranges: Vec<ByteRange>) -> Vec<ByteRange> {
    ranges.retain(|range| !range.is_empty());
    ranges.sort_by_key(|range| (range.start, range.end));
    ranges.into_iter().fold(Vec::new(), merge_range)
}

fn merge_range(mut merged: Vec<ByteRange>, next: ByteRange) -> Vec<ByteRange> {
    if let Some(last) = merged.last_mut().filter(|last| next.start <= last.end) {
        last.end = last.end.max(next.end);
    } else {
        merged.push(next);
    }
    merged
}
