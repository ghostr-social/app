mod delivery_fixture;
#[path = "delivery_focus_cooldown_retry_test/support.rs"]
mod support;

use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::start_harness;
use delivery_fixture::transient_origin::{body_count, serve};
use ghostr_delivery::delivery_events::FocusAdmission;
use support::{
    assert_no_selected_action, focused, options, wait_for_attempts, wait_for_decision_successor,
    wait_for_failures, wait_for_focus, window,
};

#[tokio::test]
async fn newly_focused_post_retries_without_reviving_other_retry_state() {
    let healthy = serve_recording("healthy", media_body(), hit_log()).await;
    let (target, target_attempts) = serve().await;
    let (unrelated, unrelated_attempts) = serve().await;
    let harness = start_harness("ghostr-focus-cooldown-retry", options());

    assert_eq!(
        harness
            .handle
            .update_focus(window(&healthy, &target, &unrelated, 1)),
        FocusAdmission::Accepted
    );
    wait_for_failures(&harness.handle, 2).await;
    assert_eq!(body_count(&target_attempts), 1, "target setup attempts");
    assert_eq!(
        body_count(&unrelated_attempts),
        1,
        "unrelated setup attempts"
    );
    assert_eq!(
        harness
            .handle
            .update_focus(focused(&healthy, &target, &unrelated, 2)),
        FocusAdmission::Accepted
    );

    wait_for_attempts(&target_attempts, 2).await;
    let target_failure = wait_for_failures(&harness.handle, 3).await;
    let settled = wait_for_decision_successor(&harness.handle, target_failure).await;
    assert_no_selected_action(&harness.handle, &settled);
    assert_eq!(
        body_count(&unrelated_attempts),
        1,
        "unfocused cooldown was cleared"
    );
    assert_eq!(
        harness
            .handle
            .update_focus(focused(&healthy, &target, &unrelated, 3)),
        FocusAdmission::Accepted
    );
    let settled = wait_for_focus(&harness.handle, 3).await;
    assert_no_selected_action(&harness.handle, &settled);
    assert_eq!(
        body_count(&target_attempts),
        2,
        "retired source was revived"
    );
    std::fs::remove_dir_all(&harness.root).ok();
}
