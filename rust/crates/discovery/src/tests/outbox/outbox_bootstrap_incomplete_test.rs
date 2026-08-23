use crate::outbox::bootstrap::OutboxBootstrap;
use crate::plan_executor::{PlanExecutor, PlanFuture, PlanPage, PlanPageFuture, PlannedRetrieval};
use crate::retrieval_types::RetrievalOutcome;
use crate::tests::outbox_support::{empty_directory, relay_list_event};
use nostr_sdk::Keys;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

struct IncompleteOutboxExecutor {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    event: nostr_sdk::Event,
}

impl PlanExecutor for IncompleteOutboxExecutor {
    fn execute(&self, _: PlannedRetrieval) -> PlanFuture {
        unreachable!("outbox bootstrap must preserve page completeness")
    }

    fn execute_page(&self, retrieval: PlannedRetrieval) -> PlanPageFuture {
        let _ = self.starts.send(retrieval);
        let event = self.event.clone();
        Box::pin(async move {
            Ok(PlanPage {
                events: vec![event],
                cursor: None,
                complete: false,
                repost_retry: Default::default(),
            })
        })
    }
}

#[tokio::test]
async fn incomplete_relay_list_page_releases_its_author_claim() {
    let author = Keys::generate();
    let (starts, mut retrievals) = mpsc::unbounded_channel();
    let executor = Arc::new(IncompleteOutboxExecutor {
        starts,
        event: relay_list_event(&author, "wss://partial.example"),
    });
    let (outcomes, mut reported) = mpsc::unbounded_channel();
    let bootstrap = OutboxBootstrap::new(executor, empty_directory(), outcomes);

    bootstrap.authors(&[author.public_key()]);
    let first = next(&mut retrievals).await;
    let outcome = timeout(Duration::from_secs(5), reported.recv())
        .await
        .expect("outcome")
        .expect("outcome channel");
    assert!(matches!(
        outcome,
        RetrievalOutcome::Completed {
            result: Ok(events), cursor: None, complete: false, ..
        } if events.len() == 1
    ));

    bootstrap.authors(&[author.public_key()]);
    let second = next(&mut retrievals).await;
    assert_eq!(first.context, second.context);
}

async fn next(retrievals: &mut mpsc::UnboundedReceiver<PlannedRetrieval>) -> PlannedRetrieval {
    timeout(Duration::from_secs(5), retrievals.recv())
        .await
        .expect("retrieval")
        .expect("retrieval channel")
}
