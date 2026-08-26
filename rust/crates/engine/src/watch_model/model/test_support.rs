use super::*;
use crate::watch_model::navigation::NavigationPrediction;

impl WatchModel {
    #[cfg(test)]
    pub(crate) const MAX_PERSISTED_GROUPS: usize = PERSISTED_GROUP_LIMIT;

    pub fn navigation(&self) -> NavigationPrediction {
        self.navigation.prediction(self.last_observed_ms)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn session_observations(&self) -> u64 {
        self.hierarchy.session_observations(self.last_observed_ms)
    }

    #[cfg(test)]
    pub(crate) fn navigation_observations(&self) -> u64 {
        self.navigation.observations()
    }

    #[cfg(test)]
    pub(crate) fn persisted_group_count(&self) -> usize {
        self.hierarchy.persistent_count()
    }

    #[cfg(test)]
    pub(crate) fn calibration_labels(&self) -> u64 {
        self.calibration.labels()
    }

    #[cfg(test)]
    pub(crate) fn calibration_error_bps(&self) -> u16 {
        self.calibration.error_bps()
    }
}
