use super::super::privacy::DecisionPrivacy;
use crate::adaptive::{
    HlsBootstrapStage, HlsBootstrapState, HlsCandidateSnapshot, HlsObjectCursor,
};
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
    #[serde(default, skip_serializing_if = "HlsObjectCursor::is_default")]
    cursor: HlsObjectCursor,
    #[serde(default, skip_serializing_if = "is_unverified")]
    player_preparation: crate::adaptive::PlayerPreparation,
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
            cursor: value.cursor,
            player_preparation: value.player_preparation,
            state: HlsState::capture(&value.state, privacy),
        }
    }
}

const fn is_zero(value: &u64) -> bool {
    *value == 0
}

fn is_unverified(value: &crate::adaptive::PlayerPreparation) -> bool {
    *value == crate::adaptive::PlayerPreparation::Unverified
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
}
