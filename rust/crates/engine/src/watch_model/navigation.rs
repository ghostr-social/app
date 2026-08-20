use serde::{Deserialize, Serialize};

const HALF_LIFE_MS: u64 = 24 * 60 * 60 * 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatchNavigation {
    Forward,
    Backward,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavigationPrediction {
    forward: f64,
    backward: f64,
    exit: f64,
}

impl NavigationPrediction {
    pub fn forward_probability(self) -> f64 {
        self.forward
    }

    pub fn backward_probability(self) -> f64 {
        self.backward
    }

    pub fn exit_probability(self) -> f64 {
        self.exit
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct NavigationState {
    forward: f64,
    backward: f64,
    exit: f64,
    observations: u64,
    last_updated_ms: u64,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            forward: 0.0,
            backward: 0.0,
            exit: 0.0,
            observations: 0,
            last_updated_ms: 0,
        }
    }
}

impl NavigationState {
    pub(crate) fn observe(&mut self, event: WatchNavigation, now_ms: u64) {
        self.decay(now_ms);
        match event {
            WatchNavigation::Forward => self.forward += 1.0,
            WatchNavigation::Backward => self.backward += 1.0,
            WatchNavigation::Exit => self.exit += 1.0,
        }
        self.observations = self.observations.saturating_add(1);
    }

    pub(crate) fn prediction(&self, now_ms: u64) -> NavigationPrediction {
        let scale = decay_scale(now_ms, self.last_updated_ms);
        let forward = 4.0 + self.forward * scale;
        let backward = 1.0 + self.backward * scale;
        let exit = 1.0 + self.exit * scale;
        let total = forward + backward + exit;
        NavigationPrediction {
            forward: forward / total,
            backward: backward / total,
            exit: exit / total,
        }
    }

    pub(crate) fn observations(&self) -> u64 {
        self.observations
    }

    pub(crate) fn sanitize(mut self) -> Self {
        self.forward = finite(self.forward);
        self.backward = finite(self.backward);
        self.exit = finite(self.exit);
        self
    }

    fn decay(&mut self, now_ms: u64) {
        let scale = decay_scale(now_ms, self.last_updated_ms);
        self.forward *= scale;
        self.backward *= scale;
        self.exit *= scale;
        self.last_updated_ms = now_ms;
    }
}

fn decay_scale(now_ms: u64, then_ms: u64) -> f64 {
    if then_ms == 0 || now_ms <= then_ms {
        return 1.0;
    }
    0.5_f64.powf(now_ms.saturating_sub(then_ms) as f64 / HALF_LIFE_MS as f64)
}

fn finite(value: f64) -> f64 {
    if value.is_finite() && value >= 0.0 {
        value.min(1_000_000.0)
    } else {
        0.0
    }
}
