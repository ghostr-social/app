use super::*;

use super::super::axiom_test_support::{fetch, FetchInput};

pub(in crate::segmented) async fn fetch_stage(
    mut input: StagedFetch<'_>,
) -> core::result::Result<FetchedObject, FetchFailure> {
    let spec = stage_spec(&input)?;
    let traffic = input.traffic.take();
    fetch(
        input.requests,
        FetchInput { spec, traffic },
        input.network_status,
        input.cancellation,
    )
    .await
}
