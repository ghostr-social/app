//! A note that advertises no digest still finalizes: the manager
//! promotes the byte-complete file instead of parking it as `.part`.

mod range_fixture;
mod support;

use range_fixture::reject::serve_failing;
use rust_lib_ghostr::engine::EngineParams;
use support::delivery::{base_params, start_harness, DeliveryOptions};
use support::delivery_items::{focus_now, sized_item, unsized_item};
use support::delivery_media::{hit_log, media_body, serve_recording};
use support::delivery_wait::{wait_for_file, wait_for_ranges};

#[tokio::test]
async fn delivery_manager_finalizes_posts_without_an_advertised_digest() {
    let origin = serve_recording("origin", media_body(), hit_log()).await;
    let hungry = serve_failing().await;
    let harness = start_harness("ghostr-delivery-unverified", short_head_options());

    let committed = sized_item("aa11", &origin, 16, 4_000);
    assert!(committed.meta.sha256.is_none(), "no imeta x on this note");
    harness.handle.update_focus(focus_now(
        vec![committed, unsized_item("bb22", &hungry)],
        0,
        5_000,
    ));

    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    wait_for_file(&harness.store.completed_path("aa11")).await;
    std::fs::remove_dir_all(&harness.root).ok();
}

/// Same head split as the commitment test: head [0,4), tail [4,16).
fn short_head_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            head_seconds: 1,
            chunk_bytes: 4,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
