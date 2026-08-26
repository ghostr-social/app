//! Primary failure still joins additive work so its relay lease can clean up.

use crate::execution::collector::axiom_test_support::collect_events;

use crate::execution::fetch::FetchedEvents;
use crate::query::search::QueryRole;
use crate::retrieval_types::PlanFailure;
use core::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;

struct DropNotice(Option<oneshot::Sender<()>>);

impl Drop for DropNotice {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

fn pending_fetch(dropped: oneshot::Sender<()>) -> JoinHandle<Result<FetchedEvents, PlanFailure>> {
    tokio::spawn(async move {
        let _notice = DropNotice(Some(dropped));
        core::future::pending().await
    })
}

#[tokio::test]
async fn primary_failure_waits_for_additive_safe_completion() {
    let primary = tokio::spawn(async { Err(PlanFailure::new("primary failed")) });
    let (release, wait) = oneshot::channel();
    let additive = tokio::spawn(async move {
        let _ = wait.await;
        Ok(FetchedEvents::fresh(Vec::new()))
    });
    let mut collection = tokio::spawn(collect_events(vec![
        (QueryRole::Primary, primary),
        (QueryRole::Additive, additive),
    ]));

    assert!(timeout(Duration::from_millis(20), &mut collection)
        .await
        .is_err());
    let _ = release.send(());
    let events = collection
        .await
        .expect("collector task")
        .expect("safe additive result");
    assert!(events.is_empty());
}

#[tokio::test]
async fn dropping_the_collector_aborts_unfinished_fetches() {
    let (first_drop, first_notice) = oneshot::channel();
    let (second_drop, second_notice) = oneshot::channel();
    let collection = tokio::spawn(collect_events(vec![
        (QueryRole::Primary, pending_fetch(first_drop)),
        (QueryRole::Additive, pending_fetch(second_drop)),
    ]));

    tokio::task::yield_now().await;
    collection.abort();
    let _ = collection.await;

    for notice in [first_notice, second_notice] {
        timeout(Duration::from_millis(100), notice)
            .await
            .expect("fetch should abort")
            .expect("drop notice");
    }
}
