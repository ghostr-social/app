use super::super::limits::ParserBudget;
use super::super::{TimedRange, TimelineError};
use super::tables::{ChunkSamples, TrackTables};
use crate::ByteRange;

pub(super) fn map_samples(
    tables: &TrackTables,
    budget: &mut ParserBudget<'_>,
    output: &mut Vec<TimedRange>,
) -> Result<(), TimelineError> {
    validate_chunks(tables, budget)?;
    budget.reserve(output, tables.sizes.len())?;
    let mut rules = ChunkRuleCursor::new(&tables.chunk_samples);
    let mut sample = 0;
    let mut media_time = 0_u64;
    for (chunk_index, chunk_start) in tables.offsets.iter().copied().enumerate() {
        budget.work(1)?;
        let count = rules.count_for(chunk_index + 1)?;
        let mut byte_offset = chunk_start;
        for _ in 0..count {
            budget.work(1)?;
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
    if sample != tables.sizes.len() || !rules.used_all() {
        return Err(TimelineError::Malformed);
    }
    Ok(())
}

struct ChunkRuleCursor<'a> {
    entries: &'a [ChunkSamples],
    index: usize,
}

impl<'a> ChunkRuleCursor<'a> {
    fn new(entries: &'a [ChunkSamples]) -> Self {
        Self { entries, index: 0 }
    }

    fn count_for(&mut self, chunk: usize) -> Result<usize, TimelineError> {
        let chunk = u32::try_from(chunk).map_err(|_conversion_error| TimelineError::Malformed)?;
        while self
            .entries
            .get(self.index + 1)
            .is_some_and(|entry| entry.first_chunk <= chunk)
        {
            self.index += 1;
        }
        let count = self
            .entries
            .get(self.index)
            .filter(|entry| entry.first_chunk <= chunk)
            .map(|entry| entry.samples_per_chunk)
            .ok_or(TimelineError::Malformed)?;
        usize::try_from(count).map_err(|_conversion_error| TimelineError::Malformed)
    }

    fn used_all(&self) -> bool {
        self.index + 1 == self.entries.len()
    }
}

#[derive(Clone, Copy)]
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

fn validate_chunks(
    tables: &TrackTables,
    budget: &mut ParserBudget<'_>,
) -> Result<(), TimelineError> {
    for entry in &tables.chunk_samples {
        budget.work(1)?;
        if entry.samples_per_chunk == 0 {
            return Err(TimelineError::Malformed);
        }
    }
    for pair in tables.chunk_samples.windows(2) {
        budget.work(1)?;
        if pair[0].first_chunk >= pair[1].first_chunk {
            return Err(TimelineError::Malformed);
        }
    }
    Ok(())
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
