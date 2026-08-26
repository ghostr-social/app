use super::prediction::{basis_points, completion, decision_mode, Prediction};
use crate::adaptive::{ActionForecast, ControlMode, HlsBootstrapStage, PlayabilitySnapshot};
use crate::origin_model::{
    MediaClass, NetworkClass, OriginContext, OriginModel, OriginQuery, RequestMethod,
};

#[derive(Clone, Copy)]
pub(super) struct HlsPredictionInput<'a> {
    pub snapshot: &'a PlayabilitySnapshot,
    pub model: &'a OriginModel,
    pub stage: HlsBootstrapStage,
    pub source: &'a str,
    pub bytes: u64,
    pub concurrency: usize,
    pub mode: ControlMode,
    pub startup_value_ms: u64,
    pub network_class: NetworkClass,
    pub completes_object: bool,
}

pub(super) fn predict(input: HlsPredictionInput<'_>) -> Prediction {
    let method = if input.stage.is_manifest() {
        RequestMethod::ManifestGet
    } else {
        RequestMethod::SegmentGet
    };
    let query = OriginQuery::new(
        input.source,
        OriginContext::new(method, input.bytes, MediaClass::Segmented)
            .with_concurrency(input.concurrency)
            .with_network(input.network_class)
            .with_observed_at_ms(input.snapshot.observed_at_ms),
    );
    let estimate = input.model.estimate(
        &query,
        input.snapshot.observed_at_ms,
        decision_mode(input.mode),
    );
    let ready = match input.stage {
        HlsBootstrapStage::FirstSegment if input.completes_object => input.startup_value_ms,
        _ => 0,
    };
    Prediction {
        forecast: ActionForecast::new(
            completion(input.bytes, &estimate),
            basis_points(estimate.success.selected),
            ready,
        ),
        uncertainty_bps: basis_points(estimate.uncertainty),
    }
}
