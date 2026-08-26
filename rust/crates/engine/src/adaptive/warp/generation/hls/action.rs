use super::{HlsCommitment, HlsValueInput};
use crate::adaptive::warp::generation::builder::Builder;
use crate::adaptive::warp::generation::hls_prediction::{predict, HlsPredictionInput};
use crate::adaptive::warp::generation::{GeneratedAction, PlannerCommand};
use crate::adaptive::{ActionKind, ActionNode, HlsBootstrapStage, HlsCandidateSnapshot};

#[derive(Clone, Copy)]
pub(super) struct Input<'a> {
    pub(super) candidate: &'a HlsCandidateSnapshot,
    pub(super) stage: HlsBootstrapStage,
    pub(super) source: &'a str,
    pub(super) commitment: HlsCommitment,
}

pub(super) fn push(builder: &mut Builder<'_>, input: Input<'_>) {
    let prediction = prediction(builder, input);
    let generated = generated_action(builder, input, prediction);
    builder.actions.push(generated);
}

fn prediction(builder: &Builder<'_>, input: Input<'_>) -> super::super::prediction::Prediction {
    predict(HlsPredictionInput {
        snapshot: builder.snapshot,
        model: builder.origins,
        stage: input.stage,
        source: input.source,
        bytes: input.commitment.expected,
        concurrency: builder
            .context
            .request_occupancy()
            .authority_count(input.source)
            .saturating_add(usize::from(input.commitment.requests > 0)),
        mode: builder.base.mode,
        startup_value_ms: input.candidate.startup_value_ms,
        network_class: builder.context.network_class(),
        completes_object: input.commitment.completes_object,
    })
}

fn generated_action(
    builder: &mut Builder<'_>,
    input: Input<'_>,
    prediction: super::super::prediction::Prediction,
) -> GeneratedAction {
    let kind = ActionKind::HlsBootstrap {
        stage: input.stage,
        cursor: input.candidate.cursor,
        maximum_bytes: input.commitment.maximum,
    };
    let node = ActionNode::new(
        builder.next_action_id(),
        input.candidate.post.clone(),
        kind,
        super::value(HlsValueInput {
            candidate: input.candidate,
            stage: input.stage,
            prediction,
            mode: builder.base.mode,
            expected_bytes: input.commitment.expected,
        }),
    )
    .with_origin(input.source)
    .with_resources(input.commitment.resources())
    .with_forecast(prediction.forecast);
    GeneratedAction {
        node,
        command: PlannerCommand::FetchHlsBootstrap {
            post: input.candidate.post.clone(),
            stage: input.stage,
            source: input.source.to_owned(),
            cursor: input.candidate.cursor,
            maximum_bytes: input.commitment.maximum,
            committed_until_ms: builder
                .snapshot
                .observed_at_ms
                .saturating_add(builder.snapshot.commitment_ms),
        },
    }
}
