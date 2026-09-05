//! Bounded local index format. Source authority belongs to the cache key,
//! never to a deserialized index or an event's claimed content hash.

use super::{MediaTimeline, StartupFootprint, TimelineError};
use serde::{Deserialize, Serialize};
use std::io::Write;

mod validation;

pub const PROFILE: &str = "mp4-v4";
pub const MAX_ENCODED_BYTES: usize = 2 * 1024 * 1024;
const FORMAT: u16 = 1;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record<T> {
    format: u16,
    profile: String,
    timeline: T,
}

/// Encodes exact sample structure without persisting a readiness assertion.
///
/// # Errors
/// Returns `ResourceLimit` if the bounded local record cannot hold the index.
pub fn encode(timeline: &MediaTimeline) -> Result<Vec<u8>, TimelineError> {
    let mut output = BoundedOutput(Vec::new());
    let record = Record {
        format: FORMAT,
        profile: PROFILE.to_owned(),
        timeline,
    };
    serde_json::to_writer(&mut output, &record).map_err(|_error| TimelineError::ResourceLimit)?;
    Ok(output.0)
}

/// Restores validated structure; callers must independently validate source authority.
///
/// # Errors
/// Rejects oversized, obsolete, malformed, or out-of-source records.
pub fn decode(bytes: &[u8], total: u64) -> Result<MediaTimeline, TimelineError> {
    if bytes.len() > MAX_ENCODED_BYTES {
        return Err(TimelineError::ResourceLimit);
    }
    let record: Record<MediaTimeline> =
        serde_json::from_slice(bytes).map_err(|_error| TimelineError::Malformed)?;
    if record.format != FORMAT || record.profile != PROFILE {
        return Err(TimelineError::Unsupported);
    }
    let mut timeline = record.timeline;
    validation::validate(&timeline, total)?;
    timeline.startup = StartupFootprint::from_timeline(&timeline);
    Ok(timeline)
}

struct BoundedOutput(Vec<u8>);

impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_ENCODED_BYTES.saturating_sub(self.0.len()) {
            return Err(std::io::Error::other("compiled index limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
