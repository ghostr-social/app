use super::calibration::CalibrationState;
use super::context::WatchContext;
use super::distribution::{DeadlineDistribution, WatchDistribution};
use super::hierarchy::WatchHierarchy;
use super::navigation::{NavigationPrediction, NavigationState, WatchNavigation};
use super::prediction::{CandidateWatchPrediction, WatchWindowPrediction};
use super::sample::{WatchSample, WatchSampleKind};
use super::state::{WatchModelState, WatchStateError, PERSISTED_GROUP_LIMIT};
use super::stats::{bin_end_ms, BINS};

#[derive(Clone, Debug, Default)]
pub struct WatchModel {
    hierarchy: WatchHierarchy,
    calibration: CalibrationState,
    navigation: NavigationState,
    revision: u64,
    change_epoch: u64,
    last_observed_ms: u64,
}

impl WatchModel {
    pub const MAX_PERSISTED_GROUPS: usize = PERSISTED_GROUP_LIMIT;

    pub fn observe(&mut self, sample: WatchSample) {
        if matches!(sample.kind, WatchSampleKind::Censored(_)) {
            return;
        }
        self.observe_calibration(&sample);
        let event = sample.kind == WatchSampleKind::Abandoned;
        let watched_ms = bounded_watch_ms(&sample);
        self.hierarchy
            .observe(&sample.context, watched_ms, event, sample.observed_at_ms);
        self.advance(sample.observed_at_ms);
    }

    pub fn observe_navigation(&mut self, event: WatchNavigation, now_ms: u64) {
        self.navigation.observe(event, now_ms);
        if matches!(event, WatchNavigation::Backward | WatchNavigation::Exit) {
            self.hierarchy.reset_session();
        }
        self.advance(now_ms);
    }

    pub fn predict(&self, context: &WatchContext, now_ms: u64) -> WatchDistribution {
        let horizon_ms = context.duration_ms.unwrap_or_else(maximum_horizon);
        let curve = prediction_times(horizon_ms)
            .into_iter()
            .map(|at_ms| {
                let raw = self.hierarchy.survival(context, at_ms, now_ms);
                (at_ms, self.calibration.calibrate(raw, now_ms))
            })
            .collect::<Vec<_>>();
        WatchDistribution::from_survival(&curve, horizon_ms)
    }

    pub fn predict_window(&self, contexts: &[WatchContext], now_ms: u64) -> WatchWindowPrediction {
        let mut start = DeadlineDistribution::immediate();
        let mut reach = 1.0;
        let forward = self.navigation.prediction(now_ms).forward_probability();
        let mut candidates = Vec::with_capacity(contexts.len());
        for context in contexts {
            let watch = self.predict(context, now_ms);
            candidates.push(CandidateWatchPrediction::new(
                watch.clone(),
                start.clone(),
                reach,
            ));
            start = start.after_watch(&watch);
            reach *= forward;
        }
        WatchWindowPrediction::new(candidates, self.revision)
    }

    pub fn navigation(&self) -> NavigationPrediction {
        self.navigation.prediction(self.last_observed_ms)
    }

    pub fn reset_session(&mut self, now_ms: u64) {
        self.hierarchy.reset_session();
        self.advance(now_ms);
    }

    pub fn state(&self) -> WatchModelState {
        WatchModelState::new(
            self.revision,
            self.change_epoch,
            self.last_observed_ms,
            self.hierarchy.persistent_state(),
            self.calibration.clone(),
            self.navigation.clone(),
        )
    }

    pub fn from_state_json(json: &str) -> Result<Self, WatchStateError> {
        let parts = WatchModelState::from_json(json)?.into_parts();
        Ok(Self {
            revision: parts.0,
            change_epoch: parts.1,
            last_observed_ms: parts.2,
            hierarchy: WatchHierarchy::from_state(parts.3, PERSISTED_GROUP_LIMIT),
            calibration: parts.4,
            navigation: parts.5,
        })
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn change_epoch(&self) -> u64 {
        self.change_epoch
    }

    pub fn session_observations(&self) -> u64 {
        self.hierarchy.session_observations(self.last_observed_ms)
    }

    pub fn navigation_observations(&self) -> u64 {
        self.navigation.observations()
    }

    pub fn persisted_group_count(&self) -> usize {
        self.hierarchy.persistent_count()
    }

    pub fn calibration_labels(&self) -> u64 {
        self.calibration.labels()
    }

    pub fn calibration_error_bps(&self) -> u16 {
        self.calibration.error_bps()
    }

    fn observe_calibration(&mut self, sample: &WatchSample) {
        let prediction = self.predict(&sample.context, sample.observed_at_ms);
        let exact_tail = sample.kind == WatchSampleKind::Abandoned;
        for at_ms in prediction_times(sample.context.duration_ms.unwrap_or_else(maximum_horizon)) {
            if at_ms > sample.watched_ms && !exact_tail {
                break;
            }
            self.calibration.observe(
                prediction.survival(at_ms),
                sample.watched_ms >= at_ms,
                sample.observed_at_ms,
            );
        }
    }

    fn advance(&mut self, now_ms: u64) {
        self.revision = self.revision.saturating_add(1);
        self.change_epoch = self.change_epoch.saturating_add(1);
        self.last_observed_ms = self.last_observed_ms.max(now_ms);
    }
}

fn bounded_watch_ms(sample: &WatchSample) -> u64 {
    sample
        .context
        .duration_ms
        .map_or(sample.watched_ms, |duration| {
            sample.watched_ms.min(duration)
        })
}

fn prediction_times(horizon_ms: u64) -> Vec<u64> {
    let mut times = (0..BINS)
        .map(bin_end_ms)
        .take_while(|at_ms| *at_ms < horizon_ms)
        .collect::<Vec<_>>();
    times.push(horizon_ms);
    times
}

fn maximum_horizon() -> u64 {
    bin_end_ms(BINS - 1)
}
