//! Watch time past the commitment threshold promotes the current
//! post's tail, finishing and finalizing the file even in hunger mode.

mod delivery_fixture;
mod range_fixture;

use delivery_fixture::items::{focus_now, sized_item, unsized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::{wait_for_file, wait_for_ranges};
use ghostr_engine::EngineParams;
use range_fixture::reject::serve_failing;

const BODY_SHA256: &str = "9f9f5111f7b27a781f1f1ddde5ebc2dd2b796bfc7365c9c28b548e564176929f";

#[tokio::test]
async fn delivery_manager_finishes_committed_posts() {
    let origin = serve_recording("origin", media_body(), hit_log()).await;
    let hungry = serve_failing().await;
    let harness = start_harness("ghostr-delivery-commit", short_head_options());

    let mut committed = sized_item("aa11", &origin, 16, 4_000);
    committed.meta.sha256 = Some(BODY_SHA256.to_owned());
    harness.handle.update_focus(focus_now(
        vec![committed, unsized_item("bb22", &hungry)],
        0,
        5_000,
    ));

    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    wait_for_file(&harness.store.completed_path("aa11")).await;
    std::fs::remove_dir_all(&harness.root).ok();
}

/// Same head split as the demand test: head [0,4), tail [4,16).
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
