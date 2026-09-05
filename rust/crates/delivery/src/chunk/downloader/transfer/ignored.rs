use super::TransferInput;
use crate::chunk::downloader::{outcome, ChunkResult, OpenedResponse, ResponseObservation};
use crate::chunk::sink::{ChunkWrite, ResponseWriteMode};

pub(super) fn range<W: ChunkWrite + ?Sized>(
    input: TransferInput<'_, '_, W>,
    range_support: Option<bool>,
) -> ChunkResult {
    let length = input.length;
    observe(input, length, range_support);
    outcome::range_ignored(length, range_support)
}

pub(super) fn bound<W: ChunkWrite + ?Sized>(
    input: TransferInput<'_, '_, W>,
    maximum_bytes: u64,
    total_bytes: u64,
    range_support: Option<bool>,
) -> anyhow::Result<ChunkResult> {
    observe(input, Some(total_bytes), range_support);
    Err(
        crate::chunk::whole_body_bound::WholeBodyBoundDiscovered::new(maximum_bytes, total_bytes)
            .into(),
    )
}

fn observe<W: ChunkWrite + ?Sized>(
    input: TransferInput<'_, '_, W>,
    total: Option<u64>,
    range_support: Option<bool>,
) {
    let response = OpenedResponse::new(
        ResponseObservation::Ignored {
            total,
            range_support,
        },
        input.generation.resumable(),
        ResponseWriteMode::Sparse,
        input.evidence,
    )
    .with_retention(input.generation.retention());
    input.traffic.response_observed(response);
}
