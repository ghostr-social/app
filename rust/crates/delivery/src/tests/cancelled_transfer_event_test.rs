use crate::chunk::downloader::ChunkResult;
use crate::manager::inflight::InFlightChunks;
use crate::manager::transfers::{chunk_event, TransferEvent};
use crate::tests::support::planned_transfer;
use ghostr_engine::adaptive::PreemptionAuthority;

#[test]
fn pre_request_cancellation_reports_the_action_terminal_ack() {
    let event = event(false);

    assert!(matches!(event, TransferEvent::ChunkDone(_)));
}

fn event(request_started: bool) -> TransferEvent {
    let transfer = planned_transfer(
        "cancelled",
        "media.example",
        PreemptionAuthority::Speculative,
    );
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(transfer.request.chunk, transfer.identity);
    chunk_event(attempt, transfer.url, Ok(result(request_started)))
}

fn result(request_started: bool) -> ChunkResult {
    ChunkResult {
        bytes_written: 0,
        range_support: None,
        range_ignored: false,
        cancelled: true,
        total_bytes: None,
        promoted: false,
        request_started,
    }
}
