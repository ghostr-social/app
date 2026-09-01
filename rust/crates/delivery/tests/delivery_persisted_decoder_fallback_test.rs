mod delivery_fixture;
mod persisted_decoder_support;

use core::time::Duration;
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::wait::wait_for_file;
use delivery_fixture::{start_harness_at, temp_directory};
use ghostr_delivery::delivery_events::{
    DeliveryCandidate, DeliveryFocus, FocusItem, PlayerPreparationDisposition,
};
use ghostr_engine::PostId;
use persisted_decoder_support::{initializing, rendition, verified_rendition};
use serde_json::json;

#[tokio::test]
async fn persisted_decoder_rejection_switches_primary_during_reconcile() {
    let root = temp_directory("ghostr-persisted-decoder-fallback");
    tokio::fs::create_dir_all(&root)
        .await
        .expect("valid test fixture");
    let high = verified_rendition("high").await;
    let low = rendition("low");
    let snapshot = json!({
        "generation": 7,
        "records": [{
            "profile": {
                "representation": high.identity().fingerprint(),
                "codec": null,
                "dimensions": null,
                "persistent": true
            },
            "result": "Unsupported"
        }],
        "revision": 1
    });
    tokio::fs::write(
        root.join("client_capability.json"),
        serde_json::to_vec(&snapshot).expect("valid test fixture"),
    )
    .await
    .expect("valid test fixture");
    let harness = start_harness_at(root.clone(), DeliveryOptions::default());
    let post = PostId::new("adaptive");
    let advertised = high.meta().clone();
    harness.handle.admit_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: advertised.clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![high, low.clone()],
        discovered_at: 1,
    });
    harness.handle.update_focus(DeliveryFocus::compatibility(
        vec![FocusItem {
            post,
            meta: advertised.clone(),
        }],
        0,
        0,
    ));

    tokio::time::timeout(Duration::from_secs(2), async {
        while harness.cache.videos().first().map(|video| &video.meta) != Some(&advertised) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("restored evidence must wait for the current player generation");
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        harness.cache.videos().first().map(|video| &video.meta),
        Some(&advertised)
    );
    wait_for_file(&root.join("adaptive.verified")).await;
    let media = harness
        .store
        .media_snapshot("adaptive")
        .await
        .expect("current media snapshot");
    let binding = media.binding().cloned().expect("current media binding");
    let report = initializing(binding, media.revision());
    let admission = harness.handle.player_preparation_admission();
    let disposition = harness
        .handle
        .confirm_player_preparation_initial(admission, report)
        .await;
    assert_eq!(disposition, PlayerPreparationDisposition::Applied);
    tokio::time::timeout(Duration::from_secs(2), async {
        while harness.cache.videos().first().map(|video| &video.meta) != Some(low.meta()) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("persisted rejection should select the advertised fallback");
    std::fs::remove_dir_all(root).ok();
}
