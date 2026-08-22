use super::limits::ParserBudget;
use super::{MediaSegment, TimelineError};

pub(super) fn canonical<'a>(
    segments: &[MediaSegment<'a>],
    budget: &mut ParserBudget<'_>,
) -> Result<Vec<MediaSegment<'a>>, TimelineError> {
    let mut ordered = budget.vector(segments.len())?;
    for segment in segments.iter().copied() {
        budget.work(1)?;
        checked_end(segment)?;
        if !segment.bytes.is_empty() {
            ordered.push(segment);
        }
    }
    ordered.sort_unstable_by_key(|segment| (segment.start, segment.bytes.len()));
    budget.work(ordered.len())?;
    validate_non_overlapping(&ordered)?;
    Ok(ordered)
}

fn validate_non_overlapping(segments: &[MediaSegment<'_>]) -> Result<(), TimelineError> {
    for pair in segments.windows(2) {
        if pair[1].start < checked_end(pair[0])? {
            return Err(TimelineError::Malformed);
        }
    }
    Ok(())
}

fn checked_end(segment: MediaSegment<'_>) -> Result<u64, TimelineError> {
    let length = u64::try_from(segment.bytes.len()).map_err(|_| TimelineError::Malformed)?;
    segment
        .start
        .checked_add(length)
        .ok_or(TimelineError::Malformed)
}
