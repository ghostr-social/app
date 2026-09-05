use super::boxes::Atom;
use super::limits::ParserBudget;
use super::{normalize, MediaTimeline, TimedRange, TimelineError};
use crate::ByteRange;

mod closure;
mod window;

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

    pub(crate) fn playable_ms(&self) -> u64 {
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
        let end = closure::startup_end(&timeline.media)?;
        let media = closure::ranges(&timeline.media, end)?;
        let headers = media_headers(&media, &timeline.media_data)?;
        let mut ranges = timeline.file_types.clone();
        ranges.push(timeline.movie?);
        ranges.extend(headers);
        ranges.extend(media);
        Self::new(ranges, end, StartupProvenance::ClassicMp4V1)
    }
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
    pub(super) inspected: Vec<ByteRange>,
    pub(super) media_data: Vec<super::boxes::MediaData>,
    pub(super) fragmented_markers: usize,
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
    let mut timeline = MediaTimeline {
        inspected: normalize(input.inspected),
        metadata: normalize(metadata),
        file_types: normalize(file_types),
        movie: input.movie,
        movie_top_level: input.movie_top_level,
        top_level_file_types: top_level_count(input.atoms, b"ftyp"),
        top_level_movies: top_level_count(input.atoms, b"moov"),
        fragmented_indexes: top_level_count(input.atoms, b"sidx")
            .saturating_add(input.fragmented_markers),
        media_data: input.media_data,
        classic_video: input.classic_video,
        startup: None,
        media: input.media,
    };
    timeline.startup = StartupFootprint::from_timeline(&timeline);
    Ok(timeline)
}

fn top_level_count(atoms: &[Atom<'_>], kind: &[u8; 4]) -> usize {
    atoms
        .iter()
        .filter(|atom| atom.is_top_level() && &atom.kind == kind)
        .count()
}

fn valid_file_type(atom: &Atom<'_>) -> bool {
    let payload = atom.payload();
    atom.start == 0 && atom.is_top_level() && payload.len() >= 8
}
