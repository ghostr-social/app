//! Primary failure still joins additive work so its relay lease can clean up.

use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::relay_plan_collector::collect_events;
use crate::discovery::search_queries::QueryRole;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;

#[tokio::test]
async fn primary_failure_waits_for_additive_fetch_completion() {
    let primary = tokio::spawn(async { Err(PlanFailure::new("primary failed")) });
    let (release, wait) = oneshot::channel();
    let additive = tokio::spawn(async move {
        let _ = wait.await;
        Ok(Vec::new())
    });
    let mut collection = tokio::spawn(collect_events(vec![
        (QueryRole::Primary, primary),
        (QueryRole::Additive, additive),
    ]));

    assert!(timeout(Duration::from_millis(20), &mut collection)
        .await
        .is_err());
    let _ = release.send(());
    let failure = collection
        .await
        .expect("collector task")
        .expect_err("primary");
    assert_eq!(failure.message, "primary failed");
}
