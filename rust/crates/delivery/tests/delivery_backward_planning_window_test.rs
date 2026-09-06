//! The delivery boundary keeps all five revisitable posts eligible for knowledge.
mod delivery_fixture;

use delivery_fixture::items::focus_now;
use delivery_fixture::plan::wait_for_current;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::{DeliveryHandle, FocusAdmission, FocusItem};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn a_long_roster_keeps_five_previous_posts_in_the_bounded_plan() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("owned origin listener");
    let origin = format!("http://{}", listener.local_addr().expect("fixture address"));
    let harness = start_harness("ghostr-five-history-window", Default::default());
    let items = (0..200).map(|index| item(index, &origin)).collect();
    assert_eq!(
        harness.handle.update_focus(focus_now(items, 50, 0)),
        FocusAdmission::Accepted,
        "the complete roster is accepted"
    );
    wait_for_current(&harness.handle, "post-50").await;
    let offsets = eligible_offsets(&harness.handle);
    assert_eq!(
        offsets.first(),
        Some(&-5),
        "fifth previous post remains preparable"
    );
    assert_eq!(
        offsets.last(),
        Some(&24),
        "future knowledge remains bounded"
    );
    assert_eq!(
        offsets.len(),
        30,
        "complete history/current/future planning window"
    );
}

fn item(index: usize, origin: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(format!("post-{index}")),
        meta: VideoMeta {
            urls: vec![format!("{origin}/{index}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    }
}

fn eligible_offsets(handle: &DeliveryHandle) -> Vec<i64> {
    let encoded = handle.decision_history_json().expect("decision evidence");
    let history: serde_json::Value = serde_json::from_str(&encoded).expect("decision JSON");
    let latest = history["decisions"]["records"]
        .as_array()
        .and_then(|records| records.last())
        .expect("focused decision");
    latest["replay_state"]["candidates"]
        .as_array()
        .expect("planning candidates")
        .iter()
        .filter(|candidate| candidate["retrieval_eligible"] == true)
        .map(|candidate| candidate["feed_offset"].as_i64().expect("feed offset"))
        .collect()
}
