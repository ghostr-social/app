use super::super::super::boxes::Atom;
use super::super::super::TimelineError;

const MAXIMUM_DECLARED_ITEMS: usize = 1_000_000;

pub(super) fn validate_count(count: usize, maximum: usize) -> Result<(), TimelineError> {
    if count == 0 || count > maximum {
        return Err(TimelineError::Malformed);
    }
    Ok(())
}

pub(super) fn require_table_extent(
    data: &[u8],
    count: usize,
    width: usize,
) -> Result<(), TimelineError> {
    let bytes = count
        .checked_mul(width)
        .and_then(|bytes| bytes.checked_add(8))
        .ok_or(TimelineError::Malformed)?;
    data.get(..bytes)
        .map(|_| ())
        .ok_or(TimelineError::Truncated)
}

pub(super) fn required<'a>(parent: &[Atom<'a>], kind: &[u8; 4]) -> Result<Atom<'a>, TimelineError> {
    parent
        .iter()
        .copied()
        .find(|atom| &atom.kind == kind)
        .ok_or(TimelineError::Malformed)
}

pub(super) fn count_at(data: &[u8], offset: usize) -> Result<usize, TimelineError> {
    let count = u32_at(data, offset)? as usize;
    if count > MAXIMUM_DECLARED_ITEMS {
        return Err(TimelineError::Malformed);
    }
    Ok(count)
}

pub(super) fn byte(data: &[u8], offset: usize) -> Result<u8, TimelineError> {
    data.get(offset).copied().ok_or(TimelineError::Truncated)
}

pub(super) fn u32_at(data: &[u8], offset: usize) -> Result<u32, TimelineError> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or(TimelineError::Truncated)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

pub(super) fn u64_at(data: &[u8], offset: usize) -> Result<u64, TimelineError> {
    let bytes = data
        .get(offset..offset + 8)
        .ok_or(TimelineError::Truncated)?;
    Ok(u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}
