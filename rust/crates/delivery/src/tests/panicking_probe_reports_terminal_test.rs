
#[path = "panicking_probe_reports_terminal_test/fixture.rs"]
mod fixture;

use crate::delivery_events::{CommandReceiver, DecisionClaim, DeliveryHandle};
use crate::manager::transfers::{spawn_probe, InternalEvent, ProbeDone, ProbeLaunch, TransferEvent};
use crate::tests::decision_log_fixture::outcome;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::representation::TransferIdentity;
use core::time::Duration;

#[tokio::test]
async fn panicking_probe_reports_a_terminal_event() {
    let tracked = fixture::tracked_head();
    let (ctx, root) = fixture::context();
    let done = receive_panic(ctx, &tracked.identity, tracked.claim).await;
    assert_panic(&done, &tracked.identity);
    resolve_panic(
        &tracked.handle,
        &tracked.commands,
        tracked.sequence,
        done.decision,
    );
    std::fs::remove_dir_all(root).expect("valid test fixture");
}

async fn receive_panic(
    ctx: crate::manager::transfers::TransferContext,
    identity: &TransferIdentity,
    claim: DecisionClaim,
) -> ProbeDone {
    let (events_sender, mut events) = tokio::sync::mpsc::unbounded_channel();
    let mut ctx = ctx;
    ctx.events = events_sender;
    spawn_probe(
        ctx,
        ProbeLaunch {
            post: identity.post().clone(),
            url: identity.source().as_str().to_owned(),
            authority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
            decision: claim,
            profile: ghostr_engine::origin_model::OriginAttemptProfile::new(
                ghostr_engine::origin_model::OriginRequestProfile::new(
                    ghostr_engine::origin_model::RequestMethod::Head,
                    0,
                    ghostr_engine::origin_model::MediaClass::Unknown,
                ),
            ),
        },
    );
    let event = tokio::time::timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("probe completion deadline")
        .expect("probe terminal event");
    let InternalEvent::Transfer(TransferEvent::ProbeDone(done)) = event else {
        panic!("probe terminal event")
    };
    *done
}

fn assert_panic(done: &ProbeDone, identity: &TransferIdentity) {
    assert_eq!(&done.observation.post, identity.post());
    assert_eq!(done.observation.url, identity.source().as_str());
    assert!(done
        .observation
        .outcome
        .as_ref()
        .expect_err("scenario must fail")
        .to_string()
        .contains("task failed"));
}

fn resolve_panic(
    handle: &DeliveryHandle,
    commands: &CommandReceiver,
    sequence: u64,
    claim: DecisionClaim,
) {
    commands
        .resolve_decision_claim(
            claim,
            DecisionOutcome::Failed {
                class: "warp_head_probe_transient".into(),
                elapsed_ms: 0,
            },
            175,
        )
        .expect("panic resolution");
    assert_eq!(
        outcome(&handle.decision_history(), sequence),
        &DecisionOutcome::Failed {
            class: "warp_head_probe_transient".into(),
            elapsed_ms: 75,
        }
    );
}
