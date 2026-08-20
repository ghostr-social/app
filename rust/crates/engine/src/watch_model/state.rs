use super::calibration::CalibrationState;
use super::hierarchy::{GroupKey, GroupState};
use super::navigation::NavigationState;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

const STATE_VERSION: u16 = 1;
pub(crate) const PERSISTED_GROUP_LIMIT: usize = 256;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchModelState {
    #[serde(default = "state_version")]
    version: u16,
    #[serde(default)]
    revision: u64,
    #[serde(default)]
    change_epoch: u64,
    #[serde(default)]
    last_observed_ms: u64,
    #[serde(default)]
    groups: Vec<GroupState>,
    #[serde(default)]
    calibration: CalibrationState,
    #[serde(default)]
    navigation: NavigationState,
}

impl WatchModelState {
    pub(crate) fn new(
        revision: u64,
        change_epoch: u64,
        last_observed_ms: u64,
        mut groups: Vec<GroupState>,
        calibration: CalibrationState,
        navigation: NavigationState,
    ) -> Self {
        groups.sort_by(group_order);
        groups.truncate(PERSISTED_GROUP_LIMIT);
        Self {
            version: STATE_VERSION,
            revision,
            change_epoch,
            last_observed_ms,
            groups,
            calibration,
            navigation,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("watch state always serializes")
    }

    pub(crate) fn from_json(json: &str) -> Result<Self, WatchStateError> {
        let state: Self = serde_json::from_str(json).map_err(WatchStateError::InvalidJson)?;
        if state.version != STATE_VERSION {
            return Err(WatchStateError::UnsupportedVersion(state.version));
        }
        Ok(state)
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        u64,
        u64,
        u64,
        Vec<GroupState>,
        CalibrationState,
        NavigationState,
    ) {
        (
            self.revision,
            self.change_epoch,
            self.last_observed_ms,
            self.groups,
            self.calibration.sanitize(),
            self.navigation.sanitize(),
        )
    }
}

#[derive(Debug)]
pub enum WatchStateError {
    InvalidJson(serde_json::Error),
    UnsupportedVersion(u16),
}

impl Display for WatchStateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(formatter, "invalid watch-model state: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported watch-model state version {version}")
            }
        }
    }
}

impl Error for WatchStateError {}

fn group_order(left: &GroupState, right: &GroupState) -> std::cmp::Ordering {
    let left_global = matches!(left.key, GroupKey::Global);
    let right_global = matches!(right.key, GroupKey::Global);
    right_global
        .cmp(&left_global)
        .then_with(|| right.stats.last_used_ms.cmp(&left.stats.last_used_ms))
        .then_with(|| left.key.cmp(&right.key))
}

const fn state_version() -> u16 {
    STATE_VERSION
}
