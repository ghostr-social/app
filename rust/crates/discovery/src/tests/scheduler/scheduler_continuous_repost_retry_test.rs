use crate::content::social_graph::SocialGraph;
use crate::feed::spec::FeedSpec;
use crate::plan_executor::{
    PlanExecutor, PlanFuture, PlanPage, PlanPageFuture, PlannedRetrieval, RepostRetryDelta,
};
use crate::retrieval_types::EventProgress;
use crate::scheduler::{start_discovery_scheduler, DiscoverySchedulerConfig};
use crate::tests::scheduler_support::{context, next_outcome, next_started};
use ghostr_engine::{adaptive::DiscoveryDemand, DataUsageLevel};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};

struct PageExecutor {
    starts: mpsc::UnboundedSender<PlannedRetrieval>,
    page: Mutex<Option<PlanPage>>,
}

impl PlanExecutor for PageExecutor {
    fn execute(&self, _: PlannedRetrieval) -> PlanFuture {
        Box::pin(std::future::pending())
    }

    fn execute_page_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        _: EventProgress,
    ) -> PlanPageFuture {
        let _ = self.starts.send(retrieval);
        let page = self.page.lock().expect("page").take();
        Box::pin(async move {
            let Some(page) = page else {
                return std::future::pending().await;
            };
            Ok(page)
        })
    }
}

#[tokio::test(start_paused = true)]
async fn continuous_following_carries_repost_into_its_next_older_page() {
    let wrapper = signed_wrapper();
    let (executor, mut started) = executor(page_with_retry(wrapper.clone()));
    let (outcomes, mut reported) = mpsc::unbounded_channel();
    let (_, demand) = watch::channel(DiscoveryDemand::Hold);
    let handle = start_discovery_scheduler(DiscoverySchedulerConfig {
        executor,
        level: DataUsageLevel::Conservative,
        demand,
        outcomes,
    });
    handle.open_feed(context("following"), following_request());
    next_started(&mut started).await;
    next_outcome(&mut reported).await;

    let retry = next_started(&mut started).await;

    assert_eq!(retry.deferred_reposts[0].id, wrapper.id);
    assert_eq!(
        retry.plan.queries[0].filter.until,
        Some(Timestamp::from(99))
    );
}

fn executor(page: PlanPage) -> (Arc<PageExecutor>, mpsc::UnboundedReceiver<PlannedRetrieval>) {
    let (starts, started) = mpsc::unbounded_channel();
    let executor = PageExecutor {
        starts,
        page: Mutex::new(Some(page)),
    };
    (Arc::new(executor), started)
}

fn page_with_retry(wrapper: Event) -> PlanPage {
    PlanPage {
        events: Vec::new(),
        cursor: Some(Timestamp::from(99)),
        complete: true,
        repost_retry: RepostRetryDelta {
            considered: Vec::new(),
            deferred: vec![wrapper],
        },
    }
}

fn following_request() -> crate::query::video_filters::DiscoveryRequest {
    let graph = SocialGraph::new(Keys::generate().public_key());
    FeedSpec::Following {
        viewer: None,
        follows: vec![Keys::generate().public_key()],
    }
    .page_request(None, &graph)
    .expect("following request")
}

fn signed_wrapper() -> Event {
    EventBuilder::new(Kind::Custom(16), "")
        .sign_with_keys(&Keys::generate())
        .unwrap()
}
