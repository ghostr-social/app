use core::time::Duration;
use std::time::SystemTime;
use tokio::time::Instant;

pub(super) struct WaitTrace {
    pub(super) started: Instant,
    pub(super) deadline: Instant,
    pub(super) wall_started: SystemTime,
    pub(super) expected: u32,
}

impl WaitTrace {
    pub(super) fn new(expected: usize, limit: Duration) -> Self {
        let started = Instant::now();
        Self {
            started,
            deadline: started + limit,
            wall_started: SystemTime::now(),
            expected: u32::try_from(expected).expect("bounded HLS fixture starts"),
        }
    }
}
