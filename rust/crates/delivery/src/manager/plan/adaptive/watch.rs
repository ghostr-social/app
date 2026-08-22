use ghostr_engine::adaptive::{
    PlannerContext, PlannerWatchEvidence, SemanticScore, TwinEpochs, ViewProbability,
};
use ghostr_engine::watch_model::{CandidateWatchPrediction, WatchContext, WatchKey, WatchModel};
use ghostr_engine::PostId;
use std::collections::BTreeMap;

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

struct CandidateInput {
    post: PostId,
    current: bool,
    duration_ms: u64,
}

impl WatchPlanningWindow {
    pub(super) fn predict(
        snapshot: &mut ghostr_engine::adaptive::PlayabilitySnapshot,
        model: &WatchModel,
    ) -> Self {
        let inputs = candidate_inputs(snapshot);
        let contexts = inputs.iter().map(watch_context).collect::<Vec<_>>();
        let prediction = model.predict_window(&contexts, snapshot.observed_at_ms);
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
                .with_semantic(post.clone(), evidence.semantic)
                .with_watch(post.clone(), evidence.watch);
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
    }
}

fn candidate_inputs(
    snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot,
) -> Vec<CandidateInput> {
    snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.feed_offset.value() >= 0)
        .map(|candidate| CandidateInput {
            post: candidate.post.clone(),
            current: candidate.feed_offset.value() == 0,
            duration_ms: candidate.duration_ms,
        })
        .collect()
}

fn watch_context(input: &CandidateInput) -> WatchContext {
    WatchContext::new(
        WatchKey::digest(input.post.as_str()),
        (input.duration_ms > 0).then_some(input.duration_ms),
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
            semantic: SemanticScore::Known(micros(reach)),
            watch,
        },
    )
}

fn basis_points(probability: f64) -> u16 {
    (probability.clamp(0.0, 1.0) * 10_000.0).round() as u16
}

fn micros(probability: f64) -> u64 {
    (probability.clamp(0.0, 1.0) * 1_000_000.0).round() as u64
}
