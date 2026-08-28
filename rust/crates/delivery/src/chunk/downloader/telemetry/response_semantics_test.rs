use super::range_compliance;
use crate::chunk::downloader::{ChunkResult, ResponseFailure, ResponseObservation};
use anyhow::Context as _;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::ByteRange;

#[test]
fn observed_range_semantics_survive_a_later_transfer_failure() {
    let bytes = ByteRange::new(0, 8);
    let request = RetrievalRequest::FetchRange {
        bytes,
        promotion: None,
    };
    let failed = Err::<ChunkResult, _>(anyhow::anyhow!("downstream failure"));

    assert_eq!(
        range_compliance(
            request,
            &failed,
            Some(ResponseObservation::Partial {
                range: bytes,
                total: Some(16),
            }),
        ),
        Some(true),
    );
    assert_eq!(
        range_compliance(
            request,
            &failed,
            Some(ResponseObservation::Ignored {
                total: Some(16),
                range_support: None,
            }),
        ),
        Some(false),
    );

    let short = Err::<ChunkResult, _>(anyhow::anyhow!("short body"))
        .context(ResponseFailure::RangeNoncompliant);
    assert_eq!(
        range_compliance(
            request,
            &short,
            Some(ResponseObservation::Partial {
                range: bytes,
                total: Some(16),
            }),
        ),
        Some(false),
    );
}
