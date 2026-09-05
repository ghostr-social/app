use super::TimelineError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PresentationTime {
    pub(super) start: i64,
    pub(super) end: i64,
    pub(super) timescale: u32,
}

impl PresentationTime {
    pub(super) fn new(start: i128, end: i128, timescale: u32) -> Result<Self, TimelineError> {
        let invalid = |_| TimelineError::Malformed;
        let value = Self {
            start: i64::try_from(start).map_err(invalid)?,
            end: i64::try_from(end).map_err(invalid)?,
            timescale,
        };
        if timescale == 0 || end <= start {
            return Err(TimelineError::Malformed);
        }
        u64::try_from((end.max(0) * 1_000 + i128::from(timescale) - 1) / i128::from(timescale))
            .map_err(invalid)?;
        Ok(value)
    }

    pub(crate) fn start_ms(self) -> u64 {
        self.millis(self.start, false)
    }
    pub(crate) fn end_ms(self) -> u64 {
        self.millis(self.end, true)
    }
    pub(super) fn end_floor_ms(self) -> u64 {
        self.millis(self.end, false)
    }

    fn millis(self, ticks: i64, ceil: bool) -> u64 {
        let rounding = if ceil {
            u128::from(self.timescale) - 1
        } else {
            0
        };
        ((ticks.max(0) as u128 * 1_000 + rounding) / u128::from(self.timescale)) as u64
    }

    pub(super) fn intersects_interval(self, start_ms: u64, end_ms: u64) -> bool {
        i128::from(self.end) * 1_000 > i128::from(start_ms) * i128::from(self.timescale)
            && i128::from(self.start) * 1_000 < i128::from(end_ms) * i128::from(self.timescale)
    }
}
