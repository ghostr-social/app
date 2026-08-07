//! Primary failure still joins additive work so its relay lease can clean up.

use crate::execution::collector::collect_events;
use crate::retrieval_types::PlanFailure;
use crate::query::search::QueryRole;
use std::time::Duration;
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

fn pending_fetch(
    dropped: oneshot::Sender<()>,
) -> JoinHandle<Result<Vec<nostr_sdk::Event>, PlanFailure>> {
    tokio::spawn(async move {
        let _notice = DropNotice(Some(dropped));
        std::future::pending().await
    })
}

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
