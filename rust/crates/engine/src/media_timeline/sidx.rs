use super::boxes::Atom;
use super::classic::samples::{scale_ceil, scale_floor};
use super::{TimedRange, TimelineError};
use crate::ByteRange;

pub(crate) fn parse(atom: &Atom<'_>) -> Result<Vec<TimedRange>, TimelineError> {
    let data = atom.payload();
    let version = byte(data, 0)?;
    let timescale = u32_at(data, 8)?;
    if timescale == 0 {
        return Err(TimelineError::Malformed);
    }
    let (mut media_time, first_offset, tail) = version_fields(data, version)?;
    let count = u16_at(data, tail + 2)? as usize;
    let mut media_start = atom
        .range()?
        .end
        .checked_add(first_offset)
        .ok_or(TimelineError::Malformed)?;
    let mut ranges = Vec::with_capacity(count);
    for index in 0..count {
        let offset = tail + 4 + index * 12;
        let reference = u32_at(data, offset)?;
        if reference >> 31 != 0 {
            return Err(TimelineError::Unsupported);
        }
        let size = u64::from(reference & 0x7fff_ffff);
        let duration = u64::from(u32_at(data, offset + 4)?);
        ranges.push(timed(ReferenceTiming {
            offset: media_start,
            size,
            start: media_time,
            duration,
            timescale,
        })?);
        media_start = media_start
            .checked_add(size)
            .ok_or(TimelineError::Malformed)?;
        media_time = media_time
            .checked_add(duration)
            .ok_or(TimelineError::Malformed)?;
    }
    Ok(ranges)
}

fn version_fields(data: &[u8], version: u8) -> Result<(u64, u64, usize), TimelineError> {
    match version {
        0 => Ok((
            u64::from(u32_at(data, 12)?),
            u64::from(u32_at(data, 16)?),
            20,
        )),
        1 => Ok((u64_at(data, 12)?, u64_at(data, 20)?, 28)),
        _ => Err(TimelineError::Unsupported),
    }
}

struct ReferenceTiming {
    offset: u64,
    size: u64,
    start: u64,
    duration: u64,
    timescale: u32,
}

fn timed(input: ReferenceTiming) -> Result<TimedRange, TimelineError> {
    if input.size == 0 || input.duration == 0 {
        return Err(TimelineError::Malformed);
    }
    let end_offset = input
        .offset
        .checked_add(input.size)
        .ok_or(TimelineError::Malformed)?;
    let end_time = input
        .start
        .checked_add(input.duration)
        .ok_or(TimelineError::Malformed)?;
    Ok(TimedRange {
        start_ms: scale_floor(input.start, input.timescale),
        end_ms: scale_ceil(end_time, input.timescale),
        bytes: ByteRange::new(input.offset, end_offset),
    })
}

fn byte(data: &[u8], offset: usize) -> Result<u8, TimelineError> {
    data.get(offset).copied().ok_or(TimelineError::Truncated)
}

fn u16_at(data: &[u8], offset: usize) -> Result<u16, TimelineError> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or(TimelineError::Truncated)?;
    Ok(u16::from_be_bytes(bytes.try_into().unwrap()))
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
