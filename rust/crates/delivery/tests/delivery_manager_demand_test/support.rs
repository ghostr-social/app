use super::delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use super::delivery_fixture::demand;
use super::delivery_fixture::items::{focus_now, sized_item};
use super::delivery_fixture::DeliveryHarness;
use super::fixture::constrained_harness;
use super::plan::wait_for_demand_plan;
use super::request::{finish, keep_alive_until, next_request};
use super::store::wait_for_stored;
use super::{DEMANDED, MEBIBYTE};
use ghostr_delivery::delivery_events::PlanEvidence;
use ghostr_delivery::playback_demand::DemandConsumer;
use ghostr_engine::adaptive::{PreemptionAuthority, MEDIA_BOOTSTRAP_PROBE_BYTES};
use ghostr_engine::ByteRange;

const POST: &str = "aa11";

pub struct DemandScenario {
    origin: ControlledOrigin,
    harness: DeliveryHarness,
    initial: Option<ActiveRequest>,
    initial_sent: usize,
    revision: u64,
}

impl DemandScenario {
    pub async fn start() -> Self {
        let mut origin = ControlledOrigin::serve(2 * MEBIBYTE).await;
        let harness = constrained_harness("ghostr-delivery-demand", MEBIBYTE);
        let item = sized_item(POST, &origin.url, 2 * MEBIBYTE, 512_000);
        harness.handle.update_focus(focus_now(vec![item], 0, 0));
        let initial = next_request(&mut origin).await;
        assert_eq!(initial.range.start, 0);
        assert!(initial.range.end >= MEDIA_BOOTSTRAP_PROBE_BYTES);
        assert!(initial.range.end <= super::CHUNK);
        let revision = harness.handle.latest_plan().expect("initial plan").revision;
        Self {
            origin,
            harness,
            initial: Some(initial),
            initial_sent: 0,
            revision,
        }
    }

    pub async fn request_demand(&mut self) -> (DemandConsumer, PlanEvidence) {
        let lease = demand::blocked(&self.harness, POST, DEMANDED).await;
        let wait = wait_for_demand_plan(&self.harness.handle, self.revision, POST, DEMANDED);
        let initial = self.initial.as_ref().expect("initial request");
        let (plan, sent) = keep_alive_until(initial, wait).await;
        self.initial_sent = sent;
        (lease, plan)
    }

    pub fn assert_demand_plan(&self, plan: &PlanEvidence) {
        let allocation = plan
            .plan
            .allocations
            .iter()
            .find(|item| item.post.as_str() == POST)
            .expect("demanded allocation");
        assert_eq!(allocation.request.requested_bytes(), DEMANDED);
        assert_eq!(allocation.authority, PreemptionAuthority::PlaybackCritical);
        let initial = self.initial.as_ref().expect("initial request");
        let retained = ByteRange::new(initial.range.start, initial.range.end);
        assert!(plan.plan.retained.iter().any(|item| {
            item.post.as_str() == POST && item.request.requested_bytes() == retained
        }));
        assert!(initial.is_open());
    }

    pub async fn complete_in_priority_order(&mut self) {
        let initial = self.initial.take().expect("initial request");
        let initial_range = initial.range.clone();
        finish(initial, self.initial_sent).await;
        wait_for_stored(&self.harness.store, POST, initial_range.clone()).await;
        let demanded = next_request(&mut self.origin).await;
        assert_eq!(demanded.range, DEMANDED.start..DEMANDED.end);
        finish(demanded, 0).await;
        wait_for_stored(&self.harness.store, POST, DEMANDED.start..DEMANDED.end).await;
        let gap = initial_range.end..DEMANDED.start;
        let missing = self.harness.store.missing_within(POST, gap.clone()).await;
        assert_eq!(missing.expect("ordinary gap"), vec![gap]);
        self.harness.handle.clear().await.expect("clear manager");
        std::fs::remove_dir_all(&self.harness.root).ok();
    }
}
