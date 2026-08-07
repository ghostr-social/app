//! Pure foundation of the media delivery engine (plan §3, Phase 1 step 3).
//! Core types shared by the catalog, focus tracking, and chunk planning.
//! No IO, no async, no clocks: everything here is deterministic and
//! table-testable.

pub mod budget;
pub mod catalog;
pub mod chunk_plan;
pub mod focus;
pub mod host_stats;
pub mod inventory_controller;
pub mod scoring;
pub mod tiers;

#[cfg(test)]
mod tests;

/// Identity of a post whose video the engine may deliver.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PostId(pub String);

impl PostId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// How a video's bytes reach the player.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DeliveryKind {
    Progressive,
    Hls,
}

/// What discovery knows about a video before any probing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoMeta {
    pub urls: Vec<String>,
    pub delivery: DeliveryKind,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub duration_ms: Option<u64>,
}

/// Half-open byte span `[start, end)` within a video file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains_offset(&self, offset: u64) -> bool {
        offset >= self.start && offset < self.end
    }
}

/// Unit of transfer work: one byte range of one post's video.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ChunkId {
    pub post: PostId,
    pub range: ByteRange,
}

/// User-facing data budget levels (mirrors the Dart `DataUsageLevel`).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DataUsageLevel {
    Conservative,
    Balanced,
    Aggressive,
}

/// The plan's tuning table (§3) in one struct so the scheduler stays
/// parameter-driven and tuning happens in tests, not on-device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EngineParams {
    pub head_seconds: u64,
    pub head_cap_bytes: u64,
    pub chunk_bytes: u64,
    pub startable_target: usize,
    pub startable_window: usize,
    pub commitment_ms: u64,
    pub emergency_buffer_s: u64,
    pub conservative_concurrency: usize,
    pub balanced_concurrency: usize,
    pub aggressive_concurrency: usize,
    pub assumed_bitrate_bps: u64,
}

impl EngineParams {
    pub fn concurrency(&self, level: DataUsageLevel) -> usize {
        match level {
            DataUsageLevel::Conservative => self.conservative_concurrency,
            DataUsageLevel::Balanced => self.balanced_concurrency,
            DataUsageLevel::Aggressive => self.aggressive_concurrency,
        }
    }
}

impl Default for EngineParams {
    fn default() -> Self {
        Self {
            head_seconds: 4,
            head_cap_bytes: 3 * 1024 * 1024,
            chunk_bytes: 1024 * 1024,
            startable_target: 4,
            startable_window: 6,
            commitment_ms: 3_000,
            emergency_buffer_s: 5,
            conservative_concurrency: 2,
            balanced_concurrency: 3,
            aggressive_concurrency: 4,
            assumed_bitrate_bps: 2_500_000,
        }
    }
}
