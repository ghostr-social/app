use super::*;

impl CalibrationDimensions {
    pub fn new(
        issuer_or_client: Option<String>,
        origin: Option<String>,
        url: Option<String>,
    ) -> Self {
        Self {
            issuer: issuer_or_client,
            client: None,
            origin,
            url,
        }
    }
}

impl CalibrationLabel {
    pub fn new(context: CalibrationContext, correct: bool, observed_at_ms: u64) -> Self {
        Self {
            context,
            correct,
            observed_at_ms,
            weight_bps: 10_000,
        }
    }
}
