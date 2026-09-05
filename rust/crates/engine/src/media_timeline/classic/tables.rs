use super::super::boxes::Atom;
use super::super::limits::ParserBudget;
use super::super::TimelineError;

mod dependencies;
mod read;
use dependencies::SampleDependencies;
use read::{byte, count_at, require_table_extent, required, u32_at, u64_at, validate_count};

pub(super) struct TrackTables {
    pub(super) dependencies: SampleDependencies,
    pub(super) timescale: u32,
    pub(super) durations: Vec<u32>,
    pub(super) sizes: Vec<u32>,
    pub(super) offsets: Vec<u64>,
    pub(super) chunk_samples: Vec<ChunkSamples>,
}

#[derive(Clone, Copy)]
pub(super) struct ChunkSamples {
    pub(super) first_chunk: u32,
    pub(super) samples_per_chunk: u32,
}

impl TrackTables {
    pub(super) fn parse(
        mdhd: &Atom<'_>,
        stbl: &[Atom<'_>],
        budget: &mut ParserBudget<'_>,
    ) -> Result<Self, TimelineError> {
        let timescale = parse_timescale(mdhd)?;
        let stsz = required(stbl, b"stsz")?;
        let sample_count = count_at(stsz.payload(), 8)?;
        budget.samples(sample_count)?;
        let durations = parse_durations(&required(stbl, b"stts")?, sample_count, budget)?;
        let sizes = parse_sizes(&stsz, sample_count, budget)?;
        let offsets = match find(stbl, b"stco") {
            Some(atom) => parse_offsets(&atom, 4, sample_count, budget)?,
            None => parse_offsets(&required(stbl, b"co64")?, 8, sample_count, budget)?,
        };
        let chunk_samples = parse_chunks(&required(stbl, b"stsc")?, offsets.len(), budget)?;
        Ok(Self {
            dependencies: SampleDependencies::parse(stbl, sample_count, budget)?,
            timescale,
            durations,
            sizes,
            offsets,
            chunk_samples,
        })
    }
}

fn parse_timescale(atom: &Atom<'_>) -> Result<u32, TimelineError> {
    let data = atom.payload();
    let offset = match byte(data, 0)? {
        0 => 12,
        1 => 20,
        _ => return Err(TimelineError::Unsupported),
    };
    let value = u32_at(data, offset)?;
    if value == 0 {
        return Err(TimelineError::Malformed);
    }
    Ok(value)
}

fn parse_durations(
    atom: &Atom<'_>,
    expected: usize,
    budget: &mut ParserBudget<'_>,
) -> Result<Vec<u32>, TimelineError> {
    let data = atom.payload();
    let count = count_at(data, 4)?;
    budget.table_work(count)?;
    let mut durations = budget.vector(expected)?;
    for index in 0..count {
        budget.work(1)?;
        let offset = 8 + index * 8;
        let samples = u32_at(data, offset)? as usize;
        let delta = u32_at(data, offset + 4)?;
        let Some(next) = durations.len().checked_add(samples) else {
            return Err(TimelineError::Malformed);
        };
        if delta == 0 || next > expected {
            return Err(TimelineError::Malformed);
        }
        budget.resize(&mut durations, next, delta)?;
    }
    (durations.len() == expected)
        .then_some(durations)
        .ok_or(TimelineError::Malformed)
}

fn parse_sizes(
    atom: &Atom<'_>,
    expected: usize,
    budget: &mut ParserBudget<'_>,
) -> Result<Vec<u32>, TimelineError> {
    let data = atom.payload();
    let fixed = u32_at(data, 4)?;
    let count = count_at(data, 8)?;
    if count != expected || count == 0 {
        return Err(TimelineError::Malformed);
    }
    budget.table_work(if fixed > 0 { 1 } else { count })?;
    let mut sizes = budget.vector(count)?;
    if fixed > 0 {
        budget.resize(&mut sizes, count, fixed)?;
        return Ok(sizes);
    }
    for index in 0..count {
        budget.work(1)?;
        sizes.push(u32_at(data, 12 + index * 4)?);
    }
    Ok(sizes)
}

fn parse_offsets(
    atom: &Atom<'_>,
    width: usize,
    maximum: usize,
    budget: &mut ParserBudget<'_>,
) -> Result<Vec<u64>, TimelineError> {
    let data = atom.payload();
    let count = count_at(data, 4)?;
    budget.table_work(count)?;
    require_table_extent(data, count, width)?;
    validate_count(count, maximum)?;
    let mut offsets = budget.vector(count)?;
    for index in 0..count {
        budget.work(1)?;
        let offset = match width {
            4 => u32_at(data, 8 + index * width).map(u64::from),
            _ => u64_at(data, 8 + index * width),
        }?;
        offsets.push(offset);
    }
    Ok(offsets)
}

fn parse_chunks(
    atom: &Atom<'_>,
    maximum: usize,
    budget: &mut ParserBudget<'_>,
) -> Result<Vec<ChunkSamples>, TimelineError> {
    let data = atom.payload();
    let count = count_at(data, 4)?;
    budget.table_work(count)?;
    require_table_extent(data, count, 12)?;
    validate_count(count, maximum)?;
    let mut entries = budget.vector(count)?;
    for index in 0..count {
        budget.work(1)?;
        let offset = 8 + index * 12;
        if u32_at(data, offset + 8)? != 1 {
            return Err(TimelineError::Unsupported);
        }
        entries.push(ChunkSamples {
            first_chunk: u32_at(data, offset)?,
            samples_per_chunk: u32_at(data, offset + 4)?,
        });
    }
    if entries.first().is_none_or(|entry| entry.first_chunk != 1) {
        return Err(TimelineError::Malformed);
    }
    Ok(entries)
}

fn find<'a>(parent: &[Atom<'a>], kind: &[u8; 4]) -> Option<Atom<'a>> {
    parent.iter().copied().find(|atom| &atom.kind == kind)
}
