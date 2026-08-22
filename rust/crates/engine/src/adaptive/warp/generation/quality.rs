use super::prediction::Prediction;
use crate::adaptive::{CandidateSnapshot, PlannerContext, PlannerQuality, PreviewAvailability};

pub(super) fn incremental(
    candidate: &CandidateSnapshot,
    context: &PlannerContext,
    prediction: Prediction,
) -> crate::adaptive::ActionForecast {
    let mut forecast = prediction.forecast;
    if forecast.ready_playback_ms == 0 {
        return forecast;
    }
    let Some(candidate_context) = context.candidate(&candidate.post) else {
        return forecast;
    };
    let expected = match candidate_context.quality {
        PlannerQuality::Unavailable => 0,
        PlannerQuality::Estimated {
            expected_micros, ..
        } => expected_micros,
    };
    let preview = match candidate_context.preview {
        PreviewAvailability::Unavailable => 0,
        PreviewAvailability::Ready { quality_micros, .. } => quality_micros,
    };
    forecast.quality_gain_micros = expected.saturating_sub(preview);
    forecast
}
