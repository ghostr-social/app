use super::{InternalEvent, ProbeDone, ProbeObservation, TransferContext, TransferEvent};
use crate::delivery_events::DecisionClaim;
use crate::probe::media::{probe, ProbeResult};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::PostId;

pub(crate) struct ProbeLaunch {
    pub post: PostId,
    pub url: String,
    pub decision: DecisionClaim,
}

/// Starts one HEAD probe under a supervisor that always reports termination.
pub(crate) fn spawn_probe(ctx: TransferContext, launch: ProbeLaunch) {
    tokio::spawn(async move {
        let events = ctx.events.clone();
        let worker = tokio::spawn(run_probe(ctx, launch.url.clone()));
        let outcome = match worker.await {
            Ok(outcome) => outcome,
            Err(error) => Err(anyhow::anyhow!("video probe task failed: {error}")),
        };
        let event = TransferEvent::ProbeDone(ProbeDone {
            observation: ProbeObservation {
                post: launch.post,
                url: launch.url,
                outcome,
            },
            decision: launch.decision,
        });
        let _ = events.send(InternalEvent::Transfer(event));
    });
}

async fn run_probe(ctx: TransferContext, url: String) -> anyhow::Result<ProbeResult> {
    let mut scratch = HostStats::new();
    probe(ctx.client.as_ref(), &url, ctx.timeouts, &mut scratch).await
}
