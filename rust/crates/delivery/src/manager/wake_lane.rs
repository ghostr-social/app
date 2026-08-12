pub(crate) const WAKE_LANES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WakeLane {
    Control = 0,
    Candidate = 1,
    Demand = 2,
    Internal = 3,
}

impl WakeLane {
    const ALL: [Self; WAKE_LANES] = [Self::Control, Self::Candidate, Self::Demand, Self::Internal];
}

#[derive(Default)]
pub(crate) struct WakeCursor {
    next: usize,
}

impl WakeCursor {
    pub(crate) fn choose(&mut self, ready: &[bool; WAKE_LANES]) -> Option<WakeLane> {
        for offset in 0..WAKE_LANES {
            let index = (self.next + offset) % WAKE_LANES;
            if ready[index] {
                let lane = WakeLane::ALL[index];
                self.observe(lane);
                return Some(lane);
            }
        }
        None
    }

    pub(crate) fn observe(&mut self, lane: WakeLane) {
        self.next = (lane as usize + 1) % WAKE_LANES;
    }
}
