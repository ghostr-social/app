//! A delayed probe cannot teach facts about a replaced source.

mod delivery_fixture;

use delivery_fixture::aba_origin::serve;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use std::time::Duration;

#[tokio::test]
async fn stale_probe_completion_releases_the_post_for_its_new_source() {
    let (old_url, old_origin) = serve(media_body()).await;
    let new_hits = hit_log();
    let new_url = serve_recording("new", media_body(), new_hits.clone()).await;
    let mut options = DeliveryOptions::default();
    options.params.conservative_concurrency = 0;
    options.params.balanced_concurrency = 0;
    options.params.aggressive_concurrency = 0;
    let harness = start_harness("ghostr-stale-probe", options);

    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("same", &old_url)], 0, 0));
    old_origin.wait_for_hits(1).await;
    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("same", &new_url)], 0, 0));
    old_origin.release_first_headers();

    wait_for_new_probe(&new_hits).await;

    assert!(hits(&new_hits).iter().any(|hit| hit == "new:HEAD:full"));
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_new_probe(log: &delivery_fixture::media::HitLog) {
    let waiting = async {
        while !hits(log).iter().any(|hit| hit.starts_with("new:HEAD")) {
            tokio::task::yield_now().await;
        }
    };
    tokio::time::timeout(Duration::from_secs(2), waiting)
        .await
        .expect("new source probe should start after stale completion");
}
