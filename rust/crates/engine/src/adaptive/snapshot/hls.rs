use super::{FeedOffset, ViewProbability};
use crate::{ActionId, PostId};
use serde::{Deserialize, Serialize};

const MIB: u64 = 1024 * 1024;
const MINIMUM_BLOCK_BYTES: u64 = 128 * 1024;
const MAXIMUM_BLOCK_BYTES: u64 = 512 * 1024;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HlsTransport {
    #[default]
    Start,
    ContinueLive {
        response: ActionId,
    },
    ResumeRange,
}

impl HlsTransport {
    pub(crate) const fn opens_request(self) -> bool {
        !matches!(self, Self::ContinueLive { .. })
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct HlsObjectCursor {
    pub attempt: u64,
    pub next_offset: u64,
    total_bytes: Option<u64>,
    pub(crate) transport: HlsTransport,
}

impl HlsObjectCursor {
    pub const fn new(
        attempt: u64,
        next_offset: u64,
        total_bytes: Option<u64>,
        transport: HlsTransport,
    ) -> Self {
        Self {
            attempt,
            next_offset,
            total_bytes,
            transport,
        }
    }

    pub fn block_bytes(self, stage: HlsBootstrapStage, requested: u64) -> Option<u64> {
        let limit = match self.total_bytes {
            Some(total) if total <= stage.maximum_bytes() => total,
            Some(_) => return None,
            None => stage.maximum_bytes(),
        };
        let remaining = limit.checked_sub(self.next_offset)?;
        (remaining > 0).then(|| stage.block_bytes(requested).min(remaining))
    }

    pub(crate) const fn completes(self, bytes: u64) -> bool {
        match self.total_bytes {
            Some(total) => self.next_offset.saturating_add(bytes) >= total,
            None => false,
        }
    }

    pub fn peak_storage_bytes(self, block_bytes: u64) -> Option<u64> {
        let end = self.next_offset.checked_add(block_bytes)?;
        match self.total_bytes {
            Some(total) if end > total => None,
            Some(total) if self.next_offset > 0 && end == total => block_bytes.checked_add(total),
            _ => Some(block_bytes),
        }
    }

    pub(crate) fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum HlsBootstrapStage {
    RootManifest,
    ChildPlaylist,
    Initialization,
    FirstSegment,
}

impl HlsBootstrapStage {
    pub const fn maximum_bytes(self) -> u64 {
        match self {
            Self::RootManifest | Self::ChildPlaylist => MIB,
            Self::Initialization | Self::FirstSegment => 8 * MIB,
        }
    }

    pub const fn is_manifest(self) -> bool {
        matches!(self, Self::RootManifest | Self::ChildPlaylist)
    }

    pub const fn block_bytes(self, requested: u64) -> u64 {
        let bounded = if requested < MINIMUM_BLOCK_BYTES {
            MINIMUM_BLOCK_BYTES
        } else if requested > MAXIMUM_BLOCK_BYTES {
            MAXIMUM_BLOCK_BYTES
        } else {
            requested
        };
        if bounded < self.maximum_bytes() {
            bounded
        } else {
            self.maximum_bytes()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HlsBootstrapState {
    Pending {
        stage: HlsBootstrapStage,
        source: String,
    },
    Active {
        action: ActionId,
        stage: HlsBootstrapStage,
        source: String,
        committed_until_ms: u64,
        cancelling: bool,
    },
    Ready,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HlsCandidateSnapshot {
    pub post: PostId,
    pub feed_offset: FeedOffset,
    pub view_probability: ViewProbability,
    pub startup_value_ms: u64,
    pub cursor: HlsObjectCursor,
    pub state: HlsBootstrapState,
}

impl HlsCandidateSnapshot {
    pub(crate) fn pending(&self) -> Option<(HlsBootstrapStage, &str)> {
        match &self.state {
            HlsBootstrapState::Pending { stage, source } => Some((*stage, source)),
            _ => None,
        }
    }

    pub(crate) const fn ready(&self) -> bool {
        matches!(self.state, HlsBootstrapState::Ready)
    }

    pub(super) fn source(&self) -> Option<&str> {
        match &self.state {
            HlsBootstrapState::Pending { source, .. }
            | HlsBootstrapState::Active { source, .. } => Some(source),
            HlsBootstrapState::Ready | HlsBootstrapState::Failed => None,
        }
    }

    pub(crate) fn active_source(&self) -> Option<&str> {
        match &self.state {
            HlsBootstrapState::Active { source, .. } => Some(source),
            _ => None,
        }
    }
}
