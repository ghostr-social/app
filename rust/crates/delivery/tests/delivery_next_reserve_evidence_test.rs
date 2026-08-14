mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording, serve_rejecting};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_engine::adaptive::{NextReserveEvidence, NextReserveInfeasibility};
use ghostr_engine::PostId;
use std::time::Duration;

#[tokio::test]
async fn manager_grants_a_servable_immediate_next_reserve() {
    let origin = serve_recording("healthy", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-next-reserve-granted", DeliveryOptions::default());

    harness.handle.update_focus(window(&origin, &origin));

    let evidence = wait_for_reserve(&harness.handle, |evidence| {
        matches!(evidence, NextReserveEvidence::Granted { .. })
    })
    .await;
    assert!(matches!(
        evidence,
        NextReserveEvidence::Granted { post, .. } if post == PostId::new("next")
    ));
    std::fs::remove_dir_all(&harness.root).ok();
}

#[tokio::test]
async fn manager_explains_an_unavailable_immediate_next_reserve() {
    let healthy = serve_recording("healthy", media_body(), hit_log()).await;
    let missing = serve_rejecting("missing", hit_log()).await;
    let harness = start_harness("ghostr-next-reserve-infeasible", DeliveryOptions::default());

    harness.handle.update_focus(window(&healthy, &missing));

    let evidence = wait_for_reserve(&harness.handle, |evidence| {
        matches!(evidence, NextReserveEvidence::Infeasible { .. })
    })
    .await;
    assert!(matches!(
        evidence,
        NextReserveEvidence::Infeasible {
            post,
            reason: NextReserveInfeasibility::NoLiveOrigin,
        } if post == PostId::new("next")
    ));
    std::fs::remove_dir_all(&harness.root).ok();
}

fn window(current: &str, next: &str) -> ghostr_delivery::delivery_events::DeliveryFocus {
    focus_now(
        vec![
            sized_item("current", current, 16, 1_000),
            sized_item("next", next, 16, 1_000),
        ],
        0,
        0,
    )
}

async fn wait_for_reserve(
    handle: &DeliveryHandle,
    expected: impl Fn(&NextReserveEvidence) -> bool,
) -> NextReserveEvidence {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(evidence) = handle
                .plan_history()
                .into_iter()
                .map(|entry| entry.plan.next_reserve)
                .find(|evidence| expected(evidence))
            {
                return evidence;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("manager did not publish immediate-next reserve evidence")
}
