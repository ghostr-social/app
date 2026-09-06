use ghostr_engine::adaptive::{
    PlannerContext, PlannerWatchEvidence, SemanticScore, TwinEpochs, ViewProbability,
};
use ghostr_engine::watch_model::{CandidateWatchPrediction, WatchModel, WatchProgress};
use ghostr_engine::PostId;
use std::collections::BTreeMap;
mod inputs;
use inputs::CandidateInput;

pub(super) struct WatchPlanningWindow {
    candidates: BTreeMap<PostId, CandidateEvidence>,
    model_epoch: u64,
}

#[derive(Clone, Copy)]
struct CandidateEvidence {
    view: ViewProbability,
    semantic: SemanticScore,
    watch: PlannerWatchEvidence,
}

impl WatchPlanningWindow {
    pub(super) fn predict(
        snapshot: &mut ghostr_engine::adaptive::PlayabilitySnapshot,
        model: &WatchModel,
        progress: WatchProgress,
        window: &crate::qoe::WatchWindow,
    ) -> Self {
        let inputs = inputs::candidates(snapshot, window);
        let contexts = inputs
            .iter()
            .map(|input| input.context.clone())
            .collect::<Vec<_>>();
        let progress = inputs
            .first()
            .filter(|input| input.current)
            .map_or_else(WatchProgress::default, |_| progress);
        let prediction =
            model.predict_remaining_window(&contexts, progress, snapshot.observed_at_ms);
        let candidates = inputs
            .into_iter()
            .zip(prediction.candidates())
            .map(|(input, prediction)| {
                candidate_evidence(input, prediction, snapshot.commitment_ms)
            })
            .collect();
        let result = Self {
            candidates,
            model_epoch: model.change_epoch(),
        };
        result.apply_snapshot(snapshot);
        result
    }

    pub(super) fn apply_context(&self, mut context: PlannerContext) -> PlannerContext {
        for (post, evidence) in &self.candidates {
            context = context
                .with_semantic(post, evidence.semantic)
                .with_watch(post, evidence.watch);
        }
        let epochs = context.epochs;
        context.with_epochs(TwinEpochs::new(
            epochs.evidence,
            self.model_epoch,
            epochs.budget,
        ))
    }

    fn apply_snapshot(&self, snapshot: &mut ghostr_engine::adaptive::PlayabilitySnapshot) {
        for candidate in &mut snapshot.candidates {
            if let Some(evidence) = self.candidates.get(&candidate.post) {
                candidate.view_probability = evidence.view;
            }
        }
        for candidate in &mut snapshot.hls_candidates {
            if let Some(evidence) = self.candidates.get(&candidate.post) {
                candidate.view_probability = evidence.view;
            }
        }
    }
}

pub(super) fn progress(
    state: &crate::manager::state::DeliveryState,
    elapsed: core::time::Duration,
) -> WatchProgress {
    state
        .playback()
        .observation()
        .and_then(|observation| core::num::NonZeroU16::new(observation.playback_rate_milli()))
        .map_or_else(
            || WatchProgress::normal(elapsed),
            |rate| WatchProgress::at_rate(elapsed, rate),
        )
}

fn candidate_evidence(
    input: CandidateInput,
    prediction: &CandidateWatchPrediction,
    commitment_ms: u64,
) -> (PostId, CandidateEvidence) {
    let reach = prediction.reach_probability();
    let play_start = prediction.play_start();
    let watch = PlannerWatchEvidence::learned(
        basis_points(reach),
        play_start.p50_ms(),
        play_start.p95_ms(),
        play_start.p99_ms(),
        basis_points(play_start.probability_by(commitment_ms)),
        input.current.then_some(commitment_ms),
    );
    (
        input.post,
        CandidateEvidence {
            view: ViewProbability::new(reach).expect("watch reach is a probability"),
            semantic: SemanticScore::Unavailable { rank: input.rank },
            watch,
        },
    )
}

fn basis_points(probability: f64) -> u16 {
    (probability.clamp(0.0, 1.0) * 10_000.0).round() as u16
}
