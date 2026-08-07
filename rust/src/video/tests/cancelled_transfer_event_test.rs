use crate::video::chunk_downloader::ChunkResult;
use crate::video::delivery_transfers::cancelled_before_request;

#[test]
fn only_pre_request_cancellation_suppresses_the_manager_event() {
    let before_request = result(false);
    let after_request = result(true);

    assert!(cancelled_before_request(&Ok(before_request)));
    assert!(!cancelled_before_request(&Ok(after_request)));
    assert!(!cancelled_before_request(&Err(anyhow::anyhow!("failure"))));
}

fn result(request_started: bool) -> ChunkResult {
    ChunkResult {
        bytes_written: 0,
        accept_ranges: false,
        cancelled: true,
        total_bytes: None,
        request_started,
    }
}
