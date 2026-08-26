use super::*;

impl CpuClock {
    pub(in super::super) const fn unavailable() -> Self {
        Self { read: || None }
    }
}
