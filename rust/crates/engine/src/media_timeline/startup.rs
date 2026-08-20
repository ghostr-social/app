use super::boxes::Atom;
use super::limits::ParserBudget;
use super::{normalize, MediaTimeline, TimedRange, TimelineError};
use crate::ByteRange;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupFootprint {
    ranges: Vec<ByteRange>,
    playable_ms: u64,
    provenance: StartupProvenance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupProvenance {
    WholeObjectV1,
    ClassicMp4V1,
}

impl StartupFootprint {
    pub(crate) fn new(
        ranges: Vec<ByteRange>,
        playable_ms: u64,
        provenance: StartupProvenance,
    ) -> Option<Self> {
        let ranges = normalize(ranges);
        (!ranges.is_empty()).then(|| Self {
            ranges,
            playable_ms: playable_ms.max(1),
            provenance,
        })
    }

    pub fn ranges(&self) -> &[ByteRange] {
        &self.ranges
    }

    pub fn playable_ms(&self) -> u64 {
        self.playable_ms
    }

    pub fn provenance(&self) -> StartupProvenance {
        self.provenance
    }

    pub(crate) fn whole(total: u64, playable_ms: u64) -> Option<Self> {
        (total > 0)
            .then(|| {
                Self::new(
                    vec![ByteRange::new(0, total)],
                    playable_ms,
                    StartupProvenance::WholeObjectV1,
                )
            })
            .flatten()
    }

    pub(super) fn from_timeline(timeline: &MediaTimeline) -> Option<Self> {
        if !timeline.classic_video || !timeline.movie_top_level || timeline.file_types.is_empty() {
            return None;
        }
        let (start, end) = first_interval(&timeline.media)?;
        let media = interval_ranges(&timeline.media, start, end);
        let headers = media_headers(&media, &timeline.media_data)?;
        let mut ranges = timeline.file_types.clone();
        ranges.push(timeline.movie?);
        ranges.extend(headers);
        ranges.extend(media);
        Self::new(
            ranges,
            end.saturating_sub(start),
            StartupProvenance::ClassicMp4V1,
        )
    }
}

fn interval_ranges(media: &[TimedRange], start: u64, end: u64) -> Vec<ByteRange> {
    media
        .iter()
        .filter(|range| range.start_ms < end && range.end_ms > start)
        .map(|range| range.bytes)
        .collect()
}

fn media_headers(media: &[ByteRange], data: &[super::boxes::MediaData]) -> Option<Vec<ByteRange>> {
    let mut headers = Vec::new();
    for range in media {
        let header = data
            .iter()
            .find(|item| contains(item.payload, *range))?
            .header;
        if !headers.contains(&header) {
            headers.push(header);
        }
    }
    Some(headers)
}

fn contains(outer: ByteRange, inner: ByteRange) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

pub(super) struct AssemblyInput<'atoms, 'bytes> {
    pub(super) atoms: &'atoms [Atom<'bytes>],
    pub(super) media_data: Vec<super::boxes::MediaData>,
    pub(super) media: Vec<TimedRange>,
    pub(super) movie: Option<ByteRange>,
    pub(super) movie_top_level: bool,
    pub(super) classic_video: bool,
}

pub(super) fn assemble(
    input: AssemblyInput<'_, '_>,
    budget: &mut ParserBudget<'_>,
) -> Result<MediaTimeline, TimelineError> {
    let mut metadata = budget.vector(input.atoms.len())?;
    let mut file_types = Vec::new();
    for atom in input.atoms {
        budget.work(1)?;
        let range = atom.range()?;
        metadata.push(range);
        match &atom.kind {
            b"ftyp" if valid_file_type(atom) => budget.push(&mut file_types, range)?,
            _ => {}
        }
    }
    Ok(MediaTimeline {
        metadata: normalize(metadata),
        file_types: normalize(file_types),
        movie: input.movie,
        movie_top_level: input.movie_top_level,
        media_data: input.media_data,
        classic_video: input.classic_video,
        media: input.media,
    })
}

fn valid_file_type(atom: &Atom<'_>) -> bool {
    let payload = atom.payload();
    atom.start == 0 && atom.is_top_level() && payload.len() >= 8
}

fn first_interval(media: &[TimedRange]) -> Option<(u64, u64)> {
    let start = media.iter().map(|range| range.start_ms).min()?;
    let end = media
        .iter()
        .filter(|range| range.start_ms == start)
        .map(|range| range.end_ms)
        .min()?;
    Some((start, end.max(start.saturating_add(1))))
}
