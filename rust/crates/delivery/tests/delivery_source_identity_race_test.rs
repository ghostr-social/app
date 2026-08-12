//! A delayed transfer cannot cross a same-post representation change.

mod delivery_fixture;

use delivery_fixture::aba_origin::serve;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{DataUsageLevel, EngineParams};
use std::time::Duration;

#[tokio::test]
async fn replacement_source_rejects_delayed_bytes_and_facts() {
    let old_bytes = b"old-byte".to_vec();
    let new_bytes = b"new!".to_vec();
    let (old_url, old_origin) = serve(old_bytes).await;
    let new_hits = hit_log();
    let new_url = serve_recording("new", new_bytes.clone(), new_hits.clone()).await;
    let harness = start_harness("ghostr-source-identity", serial_options());

    harness.handle.update_focus(focus_now(
        vec![sized_item("same", &old_url, 8, 1_000)],
        0,
        0,
    ));
    old_origin.wait_for_hits(1).await;
    harness.handle.update_focus(focus_now(
        vec![sized_item("same", &new_url, 4, 1_000)],
        0,
        0,
    ));
    wait_for_new_source(&new_hits).await;
    wait_for_ranges(&harness.store, "same", &[(0, 4)]).await;
    old_origin.release_first_headers();
    old_origin.release_body();
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(harness.store.total_len("same").await.unwrap(), Some(4));
    assert_eq!(
        harness.store.read_range("same", 0..4).await.unwrap(),
        Some(new_bytes)
    );
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_new_source(log: &delivery_fixture::media::HitLog) {
    let waiting = async {
        while !hits(log).iter().any(|hit| hit.starts_with("new:GET")) {
            tokio::task::yield_now().await;
        }
    };
    tokio::time::timeout(Duration::from_secs(1), waiting)
        .await
        .expect("replacement source must preempt the delayed transfer");
}

fn serial_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 8,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
