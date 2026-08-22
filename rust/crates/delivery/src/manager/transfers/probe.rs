use super::{InternalEvent, ProbeDone, ProbeObservation, TransferContext, TransferEvent};
use crate::delivery_events::DecisionClaim;
use crate::probe::media::{probe_observed_on_network, ObservedProbe, ProbeSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::PostId;

pub(crate) struct ProbeLaunch {
    pub post: PostId,
    pub url: String,
    pub authority: ghostr_engine::adaptive::PreemptionAuthority,
    pub decision: DecisionClaim,
}

/// Starts one HEAD probe under a supervisor that always reports termination.
pub(crate) fn spawn_probe(ctx: TransferContext, launch: ProbeLaunch) {
    tokio::spawn(async move {
        let events = ctx.events.clone();
        let network_status = ctx.network_status.clone();
        let worker = tokio::spawn(run_probe(ctx, launch.url.clone(), launch.authority));
        let observed = match worker.await {
            Ok(observed) => observed,
            Err(error) => ObservedProbe {
                outcome: Err(anyhow::anyhow!("video probe task failed: {error}")),
                concurrency: 1,
                network_class: network_status.network_class(),
            },
        };
        let event = TransferEvent::ProbeDone(ProbeDone {
            observation: ProbeObservation {
                post: launch.post,
                url: launch.url,
                outcome: observed.outcome,
                concurrency: observed.concurrency,
                network_class: observed.network_class,
            },
            decision: launch.decision,
        });
        let _ = events.send(InternalEvent::Transfer(event));
    });
}

async fn run_probe(
    ctx: TransferContext,
    url: String,
    priority: ghostr_engine::adaptive::PreemptionAuthority,
) -> ObservedProbe {
    let mut scratch = HostStats::new();
    let spec = ProbeSpec {
        requests: &ctx.requests,
        url: &url,
        priority,
        timeouts: ctx.timeouts,
    };
    probe_observed_on_network(spec, &mut scratch, &ctx.network_status).await
}
