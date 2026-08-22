use super::{FeedOffset, ViewProbability};
use crate::{ActionId, PostId};
use serde::{Deserialize, Serialize};

const MIB: u64 = 1024 * 1024;

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
    pub state: HlsBootstrapState,
}

impl HlsCandidateSnapshot {
    pub fn pending(&self) -> Option<(HlsBootstrapStage, &str)> {
        match &self.state {
            HlsBootstrapState::Pending { stage, source } => Some((*stage, source)),
            _ => None,
        }
    }

    pub const fn ready(&self) -> bool {
        matches!(self.state, HlsBootstrapState::Ready)
    }

    pub fn source(&self) -> Option<&str> {
        match &self.state {
            HlsBootstrapState::Pending { source, .. }
            | HlsBootstrapState::Active { source, .. } => Some(source),
            HlsBootstrapState::Ready | HlsBootstrapState::Failed => None,
        }
    }

    pub fn active_source(&self) -> Option<&str> {
        match &self.state {
            HlsBootstrapState::Active { source, .. } => Some(source),
            _ => None,
        }
    }
}
