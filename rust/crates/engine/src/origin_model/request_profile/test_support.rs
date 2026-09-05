use super::{OriginAttemptProfile, OriginRequestProfile};

impl OriginAttemptProfile {
    pub const fn forecast(self) -> OriginRequestProfile {
        self.forecast
    }
}
