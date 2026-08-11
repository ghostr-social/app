use super::super::boxes::{child, Atom};
use super::super::TimelineError;

const MAX_SAMPLES: usize = 1_000_000;

pub(super) struct TrackTables {
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
    pub(super) fn parse(mdhd: &Atom<'_>, stbl: &Atom<'_>) -> Result<Self, TimelineError> {
        let timescale = parse_timescale(mdhd)?;
        let durations = parse_durations(&required(stbl, b"stts")?)?;
        let sizes = parse_sizes(&required(stbl, b"stsz")?)?;
        if durations.len() != sizes.len() || sizes.is_empty() {
            return Err(TimelineError::Malformed);
        }
        let offsets = match child(stbl, b"stco")? {
            Some(atom) => parse_offsets(&atom, 4)?,
            None => parse_offsets(&required(stbl, b"co64")?, 8)?,
        };
        let chunk_samples = parse_chunks(&required(stbl, b"stsc")?)?;
        Ok(Self {
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

fn parse_durations(atom: &Atom<'_>) -> Result<Vec<u32>, TimelineError> {
    let data = atom.payload();
    let count = count_at(data, 4)?;
    let mut durations = Vec::new();
    for index in 0..count {
        let offset = 8 + index * 8;
        let samples = u32_at(data, offset)? as usize;
        let delta = u32_at(data, offset + 4)?;
        if delta == 0 || durations.len().saturating_add(samples) > MAX_SAMPLES {
            return Err(TimelineError::Malformed);
        }
        durations.resize(durations.len() + samples, delta);
    }
    Ok(durations)
}

fn parse_sizes(atom: &Atom<'_>) -> Result<Vec<u32>, TimelineError> {
    let data = atom.payload();
    let fixed = u32_at(data, 4)?;
    let count = count_at(data, 8)?;
    if fixed > 0 {
        return Ok(vec![fixed; count]);
    }
    (0..count)
        .map(|index| u32_at(data, 12 + index * 4))
        .collect()
}

fn parse_offsets(atom: &Atom<'_>, width: usize) -> Result<Vec<u64>, TimelineError> {
    let data = atom.payload();
    let count = count_at(data, 4)?;
    (0..count)
        .map(|index| match width {
            4 => u32_at(data, 8 + index * width).map(u64::from),
            _ => u64_at(data, 8 + index * width),
        })
        .collect()
}

fn parse_chunks(atom: &Atom<'_>) -> Result<Vec<ChunkSamples>, TimelineError> {
    let data = atom.payload();
    let count = count_at(data, 4)?;
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let offset = 8 + index * 12;
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

fn required<'a>(parent: &Atom<'a>, kind: &[u8; 4]) -> Result<Atom<'a>, TimelineError> {
    child(parent, kind)?.ok_or(TimelineError::Malformed)
}

fn count_at(data: &[u8], offset: usize) -> Result<usize, TimelineError> {
    let count = u32_at(data, offset)? as usize;
    if count > MAX_SAMPLES {
        return Err(TimelineError::Malformed);
    }
    Ok(count)
}

fn byte(data: &[u8], offset: usize) -> Result<u8, TimelineError> {
    data.get(offset).copied().ok_or(TimelineError::Truncated)
}

fn u32_at(data: &[u8], offset: usize) -> Result<u32, TimelineError> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or(TimelineError::Truncated)?;
    Ok(u32::from_be_bytes(bytes.try_into().unwrap()))
}

fn u64_at(data: &[u8], offset: usize) -> Result<u64, TimelineError> {
    let bytes = data
        .get(offset..offset + 8)
        .ok_or(TimelineError::Truncated)?;
    Ok(u64::from_be_bytes(bytes.try_into().unwrap()))
}
