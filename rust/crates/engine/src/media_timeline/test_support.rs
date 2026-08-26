use super::*;

struct NeverCancelled;

impl TimelineParseControl for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Parses complete metadata boxes found in the supplied absolute segments.
/// Segments may be disjoint or adjacent, but their non-empty ranges must not
/// overlap. Input order does not affect the result.
///
/// # Errors
///
/// Returns a timeline error when segment geometry, MP4 structure, or parser resource limits are
/// invalid.
pub fn parse_mp4_segments(segments: &[MediaSegment<'_>]) -> Result<MediaTimeline, TimelineError> {
    parse_mp4_segments_with_control(segments, &NeverCancelled)
}
