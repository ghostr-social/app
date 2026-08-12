use super::super::{TimedRange, TimelineError};
use super::tables::{ChunkSamples, TrackTables};
use crate::ByteRange;

pub(super) fn map_samples(tables: TrackTables) -> Result<Vec<TimedRange>, TimelineError> {
    validate_chunks(&tables)?;
    let mut output = Vec::with_capacity(tables.sizes.len());
    let mut sample = 0;
    let mut media_time = 0_u64;
    for (chunk_index, chunk_start) in tables.offsets.iter().copied().enumerate() {
        let count = samples_in_chunk(&tables.chunk_samples, chunk_index + 1)?;
        let mut byte_offset = chunk_start;
        for _ in 0..count {
            let size = *tables.sizes.get(sample).ok_or(TimelineError::Malformed)?;
            let duration = tables.durations[sample];
            output.push(timed(SampleTiming {
                offset: byte_offset,
                size,
                start: media_time,
                duration,
                timescale: tables.timescale,
            })?);
            byte_offset = byte_offset
                .checked_add(u64::from(size))
                .ok_or(TimelineError::Malformed)?;
            media_time = media_time
                .checked_add(u64::from(duration))
                .ok_or(TimelineError::Malformed)?;
            sample += 1;
        }
    }
    if sample != tables.sizes.len() {
        return Err(TimelineError::Malformed);
    }
    Ok(output)
}

struct SampleTiming {
    offset: u64,
    size: u32,
    start: u64,
    duration: u32,
    timescale: u32,
}

fn timed(input: SampleTiming) -> Result<TimedRange, TimelineError> {
    if input.size == 0 {
        return Err(TimelineError::Malformed);
    }
    let end = input
        .start
        .checked_add(u64::from(input.duration))
        .ok_or(TimelineError::Malformed)?;
    let byte_end = input
        .offset
        .checked_add(u64::from(input.size))
        .ok_or(TimelineError::Malformed)?;
    Ok(TimedRange {
        start_ms: scale_floor(input.start, input.timescale),
        end_ms: scale_ceil(end, input.timescale),
        bytes: ByteRange::new(input.offset, byte_end),
    })
}

fn samples_in_chunk(entries: &[ChunkSamples], chunk: usize) -> Result<usize, TimelineError> {
    let chunk = u32::try_from(chunk).map_err(|_| TimelineError::Malformed)?;
    let count = entries
        .iter()
        .take_while(|entry| entry.first_chunk <= chunk)
        .last()
        .map(|entry| entry.samples_per_chunk)
        .ok_or(TimelineError::Malformed)?;
    if count == 0 {
        return Err(TimelineError::Malformed);
    }
    Ok(count as usize)
}

fn validate_chunks(tables: &TrackTables) -> Result<(), TimelineError> {
    let valid = tables
        .chunk_samples
        .windows(2)
        .all(|pair| pair[0].first_chunk < pair[1].first_chunk);
    match valid {
        true => Ok(()),
        false => Err(TimelineError::Malformed),
    }
}

pub(crate) fn scale_floor(value: u64, timescale: u32) -> u64 {
    value.saturating_mul(1_000) / u64::from(timescale)
}

pub(crate) fn scale_ceil(value: u64, timescale: u32) -> u64 {
    value
        .saturating_mul(1_000)
        .saturating_add(u64::from(timescale) - 1)
        / u64::from(timescale)
}
