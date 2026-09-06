//! Public manager fixture for conditional watch deadlines.
mod evidence;
pub mod playback;

use super::items::{focus_now, sized_item};
use super::plan::wait_for_plan;
use super::{start_harness, DeliveryHarness};
use ghostr_delivery::delivery_events::{
    DeliveryHandle, FocusAdmission, FocusGeneration, FocusItem, PlanEvidence,
};

pub struct WatchFixture {
    _listener: tokio::net::TcpListener,
    harness: DeliveryHarness,
    items: Vec<FocusItem>,
}

impl WatchFixture {
    pub async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("owned origin listener");
        let url = format!(
            "http://{}/video.mp4",
            listener.local_addr().expect("address")
        );
        Self {
            _listener: listener,
            harness: start_harness("ghostr-remaining-watch", Default::default()),
            items: vec![
                sized_item("current", &url, 16, 8_000),
                sized_item("next", &url, 16, 12_000),
            ],
        }
    }

    pub fn handle(&self) -> &DeliveryHandle {
        &self.harness.handle
    }

    pub fn with_hls_current(mut self) -> Self {
        self.items[0].meta.delivery = ghostr_engine::DeliveryKind::Hls;
        self.items[0].meta.urls[0] =
            self.items[0].meta.urls[0].replace("video.mp4", "current.m3u8");
        self
    }

    pub fn with_hls_middle(mut self) -> Self {
        let mut middle = self.items[1].clone();
        middle.post = ghostr_engine::PostId::new("middle");
        middle.meta.delivery = ghostr_engine::DeliveryKind::Hls;
        middle.meta.urls[0] = middle.meta.urls[0].replace("video.mp4", "middle.m3u8");
        self.items.insert(1, middle);
        self
    }

    pub async fn focus(&self, watched_ms: u64, generation: u64) -> PlanEvidence {
        let mut focus = focus_now(self.items.clone(), 0, watched_ms);
        focus.generation = FocusGeneration::try_new(generation).expect("focus generation");
        assert_eq!(
            self.handle().update_focus(focus),
            FocusAdmission::Accepted,
            "focus admitted"
        );
        wait_for_plan(self.handle(), 0, |plan| {
            plan.focus_generation == Some(generation) && plan.decision_sequence.is_some()
        })
        .await
    }

    pub fn next_deadline(&self, plan: &PlanEvidence) -> u64 {
        self.deadline_at(plan, 1)
    }

    pub fn deadline_at(&self, plan: &PlanEvidence, offset: i32) -> u64 {
        evidence::deadline_at(self.handle(), plan, offset)
    }
}
