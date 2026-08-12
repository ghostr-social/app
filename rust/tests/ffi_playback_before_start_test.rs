use rust_lib_ghostr::api::playback_control::ffi_report_playback;
use rust_lib_ghostr::api::playback_types::{FfiPlaybackObservation, FfiPlaybackPhase};

#[tokio::test]
async fn playback_report_fails_closed_before_the_engine_starts() {
    let error = ffi_report_playback(observation())
        .await
        .expect_err("engine is not running");

    assert!(error.to_string().contains("not initialized"));
}

fn observation() -> FfiPlaybackObservation {
    FfiPlaybackObservation {
        post_id: "clip".to_owned(),
        generation: 1,
        sequence: 1,
        phase: FfiPlaybackPhase::Playing,
        position_ms: 0,
        buffered_extent_ms: 1_000,
        playback_rate_milli: 1_000,
    }
}
