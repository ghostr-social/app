mod delivery_fixture;

use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::playback::playing;
use delivery_fixture::start_harness;
use std::time::Duration;

#[tokio::test]
async fn swipe_cancels_speculation_and_fills_the_reserved_playback_slot() {
    let mut origins = Origins::serve().await;
    let mut options = DeliveryOptions::default();
    options.params.balanced_concurrency = 4;
    let harness = start_harness("ghostr-swipe-replenishment", options);
    let current = origins.item(0);
    seed_range(&harness.store, &current, 0, &[7; 32]).await;
    harness
        .handle
        .update_focus(focus_now(origins.items(current), 0, 0));
    harness
        .handle
        .report_playback(playing("p0", Duration::from_secs(20)));
    let active = origins.first_window().await;

    harness
        .handle
        .update_focus(focus_now(origins.all_items(), 3, 0));
    harness
        .handle
        .report_playback(playing("p3", Duration::from_secs(20)));

    let current = next_request("p3 critical", &mut origins.posts[3]).await;
    wait_cancelled(&active[0]).await;
    assert!(current.send_byte().await, "new current request is live");
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

struct Origins {
    posts: Vec<ControlledOrigin>,
}

impl Origins {
    async fn serve() -> Self {
        let mut posts = Vec::new();
        for _ in 0..7 {
            posts.push(ControlledOrigin::serve(32).await);
        }
        Self { posts }
    }

    fn item(&self, index: usize) -> ghostr_delivery::delivery_events::FocusItem {
        sized_item(id(index), &self.posts[index].url, 32, 4_000)
    }

    fn items(
        &self,
        first: ghostr_delivery::delivery_events::FocusItem,
    ) -> Vec<ghostr_delivery::delivery_events::FocusItem> {
        std::iter::once(first)
            .chain((1..7).map(|index| self.item(index)))
            .collect()
    }

    fn all_items(&self) -> Vec<ghostr_delivery::delivery_events::FocusItem> {
        (0..7).map(|index| self.item(index)).collect()
    }

    async fn first_window(&mut self) -> Vec<ActiveRequest> {
        let mut requests = Vec::new();
        for index in 2..=4 {
            requests.push(next_request(id(index), &mut self.posts[index]).await);
        }
        requests
    }
}

fn id(index: usize) -> &'static str {
    ["p0", "p1", "p2", "p3", "p4", "p5", "p6"][index]
}

async fn next_request(label: &str, origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .unwrap_or_else(|_| panic!("{label} starts in time"))
}

async fn wait_cancelled(request: &ActiveRequest) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while request.send_byte().await {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("stale request is cancelled");
}
