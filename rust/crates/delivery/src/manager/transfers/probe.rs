use super::{InternalEvent, ProbeDone, ProbeObservation, TransferContext, TransferEvent};
use crate::delivery_events::DecisionClaim;
use crate::probe::media::{probe, ObservedProbe, ProbeSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::PostId;

pub(crate) struct ProbeLaunch {
    pub post: PostId,
    pub url: String,
    pub authority: ghostr_engine::adaptive::PreemptionAuthority,
    pub decision: DecisionClaim,
    pub profile: ghostr_engine::origin_model::OriginAttemptProfile,
}

/// Starts one HEAD probe under a supervisor that always reports termination.
pub(crate) fn spawn_probe(ctx: TransferContext, launch: ProbeLaunch) -> tokio::task::AbortHandle {
    let events = ctx.events.clone();
    let worker = tokio::spawn(run_probe(
        ctx,
        launch.url.clone(),
        launch.authority,
        launch.profile,
    ));
    let abort = worker.abort_handle();
    tokio::spawn(async move {
        let observed = match worker.await {
            Ok(observed) => observed,
            Err(error) => ObservedProbe {
                outcome: Err(anyhow::anyhow!("video probe task failed: {error}")),
                attempt_context: None,
            },
        };
        let event = TransferEvent::ProbeDone(Box::new(ProbeDone {
            observation: ProbeObservation {
                post: launch.post,
                url: launch.url,
                outcome: observed.outcome,
                attempt_context: observed.attempt_context,
            },
            decision: launch.decision,
        }));
        let _ = events.send(InternalEvent::Transfer(event));
    });
    abort
}

async fn run_probe(
    ctx: TransferContext,
    url: String,
    priority: ghostr_engine::adaptive::PreemptionAuthority,
    profile: ghostr_engine::origin_model::OriginAttemptProfile,
) -> ObservedProbe {
    let mut scratch = HostStats::new();
    let spec = ProbeSpec {
        requests: &ctx.requests,
        url: &url,
        priority,
        timeouts: ctx.timeouts,
        network: Some(&ctx.network_status),
        profile,
    };
    probe(spec, &mut scratch).await
}
