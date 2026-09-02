use super::*;

impl ActionForecast {
    pub(crate) const fn with_quality(mut self, gain_micros: u64) -> Self {
        self.quality_gain_micros = gain_micros;
        self
    }
}
