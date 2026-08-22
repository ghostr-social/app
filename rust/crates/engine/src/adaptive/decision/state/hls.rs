use super::super::privacy::DecisionPrivacy;
use crate::adaptive::{
    FeedOffset, HlsBootstrapStage, HlsBootstrapState, HlsCandidateSnapshot, ViewProbability,
};
use crate::{ActionId, PostId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct HlsCandidateState {
    post: String,
    feed_offset: i32,
    view_probability: f64,
    startup_value_ms: u64,
    #[serde(
        default,
        rename = "segmented_storage_available_bytes",
        skip_serializing_if = "is_zero"
    )]
    legacy_segmented_storage_available_bytes: u64,
    state: HlsState,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum HlsState {
    Pending {
        stage: HlsBootstrapStage,
        source: String,
    },
    Active {
        action_id: u64,
        stage: HlsBootstrapStage,
        source: String,
        committed_until_ms: u64,
        cancelling: bool,
    },
    Ready,
    Failed,
}

impl HlsCandidateState {
    pub(super) fn capture(value: &HlsCandidateSnapshot, privacy: &DecisionPrivacy) -> Self {
        Self {
            post: privacy.post(value.post.as_str()),
            feed_offset: value.feed_offset.value(),
            view_probability: value.view_probability.value(),
            startup_value_ms: value.startup_value_ms,
            legacy_segmented_storage_available_bytes: 0,
            state: HlsState::capture(&value.state, privacy),
        }
    }

    pub(super) fn snapshot(&self) -> HlsCandidateSnapshot {
        HlsCandidateSnapshot {
            post: PostId::new(&self.post),
            feed_offset: FeedOffset::new(self.feed_offset),
            view_probability: ViewProbability::new(self.view_probability)
                .expect("captured probability remains valid"),
            startup_value_ms: self.startup_value_ms,
            state: self.state.snapshot(),
        }
    }
}

const fn is_zero(value: &u64) -> bool {
    *value == 0
}

impl HlsState {
    fn capture(value: &HlsBootstrapState, privacy: &DecisionPrivacy) -> Self {
        match value {
            HlsBootstrapState::Pending { stage, source } => Self::Pending {
                stage: *stage,
                source: privacy.source(source),
            },
            HlsBootstrapState::Active {
                action,
                stage,
                source,
                committed_until_ms,
                cancelling,
            } => Self::Active {
                action_id: action.value(),
                stage: *stage,
                source: privacy.source(source),
                committed_until_ms: *committed_until_ms,
                cancelling: *cancelling,
            },
            HlsBootstrapState::Ready => Self::Ready,
            HlsBootstrapState::Failed => Self::Failed,
        }
    }

    fn snapshot(&self) -> HlsBootstrapState {
        match self {
            Self::Pending { stage, source } => HlsBootstrapState::Pending {
                stage: *stage,
                source: source.clone(),
            },
            Self::Active {
                action_id,
                stage,
                source,
                committed_until_ms,
                cancelling,
            } => HlsBootstrapState::Active {
                action: ActionId::new(*action_id),
                stage: *stage,
                source: source.clone(),
                committed_until_ms: *committed_until_ms,
                cancelling: *cancelling,
            },
            Self::Ready => HlsBootstrapState::Ready,
            Self::Failed => HlsBootstrapState::Failed,
        }
    }
}
