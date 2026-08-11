//! Turns a video's metadata into concrete byte-range work items:
//! the startability head, tail chunks, and the moov tail probe.

#[cfg(test)]
use crate::VideoMeta;
use crate::{ByteRange, EngineParams};

/// How much of the file end to fetch when moov may sit at the end.
pub(crate) const TAIL_PROBE_BYTES: u64 = 256 * 1024;

/// What the planner needs to know about one video. Callers refine
/// `size_bytes` with probed values (see `CatalogEntry::total_bytes`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanInput {
    pub(crate) size_bytes: Option<u64>,
    pub(crate) duration_ms: Option<u64>,
    pub(crate) bitrate_bps: u64,
}

/// The chunk layout for one video under the current parameters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChunkPlan {
    size_bytes: Option<u64>,
    head_bytes: u64,
    chunk_bytes: u64,
    needs_tail_probe: bool,
}

impl ChunkPlan {
    #[cfg(test)]
    pub(crate) fn for_meta(meta: &VideoMeta, bitrate_bps: u64, params: &EngineParams) -> Self {
        Self::from_input(
            PlanInput {
                size_bytes: meta.size_bytes,
                duration_ms: meta.duration_ms,
                bitrate_bps,
            },
            params,
        )
    }

    #[cfg(test)]
    fn from_input(input: PlanInput, params: &EngineParams) -> Self {
        Self::from_input_with_head_seconds(input, params, params.head_seconds)
    }

    pub(crate) fn from_input_with_head_seconds(
        input: PlanInput,
        params: &EngineParams,
        head_seconds: u64,
    ) -> Self {
        Self {
            size_bytes: input.size_bytes,
            head_bytes: head_bytes(input, params, head_seconds),
            chunk_bytes: params.chunk_bytes.max(1),
            needs_tail_probe: input.duration_ms.is_none(),
        }
    }

    /// Bytes that make the video startable (plan §3 head budget).
    pub(crate) fn head_bytes(&self) -> u64 {
        self.head_bytes
    }

    /// Duration (and therefore moov placement) is unknown: fetch the
    /// tail probe range before declaring the video startable.
    pub(crate) fn needs_tail_probe(&self) -> bool {
        self.needs_tail_probe
    }

    /// The head split into chunk-sized ranges, in fetch order.
    pub(crate) fn head_ranges(&self) -> Vec<ByteRange> {
        split(0, self.head_bytes, self.chunk_bytes)
    }

    /// Chunks from the end of the head to the end of the file. Empty
    /// while the file size is unknown.
    pub(crate) fn tail_ranges(&self) -> Vec<ByteRange> {
        match self.size_bytes {
            Some(size) if size > self.head_bytes => split(self.head_bytes, size, self.chunk_bytes),
            _ => Vec::new(),
        }
    }

    /// The final ~256 KiB, wanted only when a tail probe is needed and
    /// the file size is known (probe for size first otherwise).
    pub(crate) fn tail_probe_range(&self) -> Option<ByteRange> {
        if !self.needs_tail_probe {
            return None;
        }
        let size = self.size_bytes?;
        Some(ByteRange::new(size.saturating_sub(TAIL_PROBE_BYTES), size))
    }

    /// First planned chunk (head first, then tail) not fully covered by
    /// the ranges already on disk.
    pub(crate) fn next_missing_chunk(&self, have: &[ByteRange]) -> Option<ByteRange> {
        self.head_ranges()
            .into_iter()
            .chain(self.tail_ranges())
            .find_map(|range| first_missing_within(range, have))
    }
}

/// Head budget: `head_seconds` at the estimated bitrate, capped, and
/// never more than the whole file.
fn head_bytes(input: PlanInput, params: &EngineParams, head_seconds: u64) -> u64 {
    let ideal = head_seconds.saturating_mul(input.bitrate_bps) / 8;
    let capped = ideal.min(params.head_cap_bytes);
    match input.size_bytes {
        Some(size) => capped.min(size),
        None => capped,
    }
}

fn split(start: u64, end: u64, chunk: u64) -> Vec<ByteRange> {
    let mut ranges = Vec::new();
    let mut cursor = start;
    while cursor < end {
        let next = cursor.saturating_add(chunk).min(end);
        ranges.push(ByteRange::new(cursor, next));
        cursor = next;
    }
    ranges
}

fn first_missing_within(range: ByteRange, have: &[ByteRange]) -> Option<ByteRange> {
    let mut cursor = range.start;
    while cursor < range.end {
        match furthest_reach(cursor, have) {
            Some(reach) => cursor = reach.min(range.end),
            None => {
                return Some(ByteRange::new(
                    cursor,
                    next_present(cursor, range.end, have),
                ))
            }
        }
    }
    None
}

fn furthest_reach(cursor: u64, have: &[ByteRange]) -> Option<u64> {
    have.iter()
        .filter(|range| range.start <= cursor && range.end > cursor)
        .map(|range| range.end)
        .max()
}

fn next_present(cursor: u64, end: u64, have: &[ByteRange]) -> u64 {
    have.iter()
        .filter(|range| range.start > cursor)
        .map(|range| range.start)
        .min()
        .unwrap_or(end)
        .min(end)
}
