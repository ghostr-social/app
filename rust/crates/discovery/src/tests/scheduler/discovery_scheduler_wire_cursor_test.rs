use crate::plan_executor::{PlanExecutor, PlanFuture, PlanPage, PlanPageFuture, PlannedRetrieval};
use crate::retrieval_types::EventProgress;
use crate::scheduler::{start_discovery_scheduler, DiscoverySchedulerConfig};
use crate::tests::scheduler_support::{context, next_outcome, next_started, request};
use ghostr_engine::{adaptive::DiscoveryDemand, DataUsageLevel};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};

struct CursorExecutor {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    pages: Mutex<VecDeque<PlanPage>>,
}

impl PlanExecutor for CursorExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let _ = self.starts.send(retrieval);
        let page = self.pages.lock().expect("pages").pop_front();
        Box::pin(async move {
            match page {
                Some(page) => Ok(page.events),
                None => std::future::pending().await,
            }
        })
    }

    fn execute_page_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        _progress: EventProgress,
    ) -> PlanPageFuture {
        let _ = self.starts.send(retrieval);
        let page = self.pages.lock().expect("pages").pop_front();
        Box::pin(async move {
            match page {
                Some(page) => Ok(page),
                None => std::future::pending().await,
            }
        })
    }
}

#[tokio::test]
async fn scheduler_uses_the_shallowest_wire_filter_cursor() {
    let events = vec![
        event(Kind::Custom(21), 80),
        event(Kind::TextNote, 100),
        event(Kind::TextNote, 50),
        event(Kind::Custom(1063), 70),
    ];
    let (starts, mut started) = mpsc::unbounded_channel();
    let executor = CursorExecutor {
        starts,
        pages: Mutex::new(
            vec![PlanPage {
                events,
                cursor: Some(Timestamp::from(99)),
            }]
            .into(),
        ),
    };
    let (outcomes, mut reported) = mpsc::unbounded_channel();
    let (_, demand) = watch::channel(DiscoveryDemand::Hold);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor: Arc::new(executor),
        level: DataUsageLevel::Conservative,
        demand,
        outcomes,
    });
    let mut query = request();
    query.search_query = Some("ghost".to_owned());
    handle.open_feed(context("search"), query);

    next_started(&mut started).await;
    next_outcome(&mut reported).await;
    let older = next_started(&mut started).await;

    assert_eq!(
        older.plan.queries[0].filter.until,
        Some(Timestamp::from(99))
    );
}

fn event(kind: Kind, created_at: u64) -> Event {
    EventBuilder::new(kind, "event")
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed")
}
