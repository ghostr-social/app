use crate::adaptive::HlsBootstrapStage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedHlsBootstrapStage {
    RootManifest,
    ChildPlaylist,
    Initialization,
    FirstSegment,
}

impl From<HlsBootstrapStage> for RecordedHlsBootstrapStage {
    fn from(value: HlsBootstrapStage) -> Self {
        match value {
            HlsBootstrapStage::RootManifest => Self::RootManifest,
            HlsBootstrapStage::ChildPlaylist => Self::ChildPlaylist,
            HlsBootstrapStage::Initialization => Self::Initialization,
            HlsBootstrapStage::FirstSegment => Self::FirstSegment,
        }
    }
}

impl From<RecordedHlsBootstrapStage> for HlsBootstrapStage {
    fn from(value: RecordedHlsBootstrapStage) -> Self {
        match value {
            RecordedHlsBootstrapStage::RootManifest => Self::RootManifest,
            RecordedHlsBootstrapStage::ChildPlaylist => Self::ChildPlaylist,
            RecordedHlsBootstrapStage::Initialization => Self::Initialization,
            RecordedHlsBootstrapStage::FirstSegment => Self::FirstSegment,
        }
    }
}
