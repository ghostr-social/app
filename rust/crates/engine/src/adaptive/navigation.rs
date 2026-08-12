use super::{NavigationSnapshot, ViewProbability};
use std::collections::VecDeque;

const RATE_WINDOW_MS: u64 = 10_000;
const RATE_SCALE: u64 = 60_000 / RATE_WINDOW_MS;
const MAX_EVENTS: usize = 64;
const FORWARD_PRIOR: u32 = 4;
const BACKWARD_PRIOR: u32 = 1;
const DISTANCE_DECAY: f64 = 0.82;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FeedOffset(i32);

impl FeedOffset {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> i32 {
        self.0
    }

    pub fn magnitude(self) -> u32 {
        self.0.unsigned_abs()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NavigationEvent {
    direction: NavigationDirection,
    observed_at_ms: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NavigationHistory {
    events: VecDeque<NavigationEvent>,
}

impl NavigationHistory {
    pub fn record(&mut self, direction: NavigationDirection, observed_at_ms: u64) {
        self.discard_stale(observed_at_ms);
        if self.events.len() == MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(NavigationEvent {
            direction,
            observed_at_ms,
        });
    }

    pub fn snapshot(&self, observed_at_ms: u64) -> NavigationSnapshot {
        NavigationSnapshot {
            forward_swipes_per_minute: self.rate(NavigationDirection::Forward, observed_at_ms),
            backward_swipes_per_minute: self.rate(NavigationDirection::Backward, observed_at_ms),
        }
    }

    fn discard_stale(&mut self, observed_at_ms: u64) {
        while self.events.front().is_some_and(|event| {
            observed_at_ms.saturating_sub(event.observed_at_ms) > RATE_WINDOW_MS
        }) {
            self.events.pop_front();
        }
    }

    fn rate(&self, direction: NavigationDirection, observed_at_ms: u64) -> u16 {
        let count = self
            .events
            .iter()
            .filter(|event| event.direction == direction)
            .filter(|event| observed_at_ms.saturating_sub(event.observed_at_ms) <= RATE_WINDOW_MS)
            .count() as u64;
        count.saturating_mul(RATE_SCALE).min(u64::from(u16::MAX)) as u16
    }
}

impl NavigationSnapshot {
    pub fn view_probability(self, offset: FeedOffset) -> ViewProbability {
        if offset.value() == 0 {
            return ViewProbability::new(1.0).expect("one is a probability");
        }
        let forward = FORWARD_PRIOR + u32::from(self.forward_swipes_per_minute);
        let backward = BACKWARD_PRIOR + u32::from(self.backward_swipes_per_minute);
        let directional = match offset.value().is_positive() {
            true => forward as f64,
            false => backward as f64,
        } / f64::from(forward + backward);
        let exponent = offset.magnitude().saturating_sub(1).min(64) as i32;
        ViewProbability::new(directional * DISTANCE_DECAY.powi(exponent))
            .expect("directional probability is bounded")
    }
}
