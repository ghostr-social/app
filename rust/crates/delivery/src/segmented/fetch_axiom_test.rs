use super::*;

pub(super) use open::open;

pub(in super::super) use staged::axiom_test_support::fetch_stage;

pub(super) struct FetchInput<'a> {
    pub(super) spec: FetchSpec<'a>,
    pub(super) traffic: Option<SegmentedTraffic>,
}

pub(super) async fn fetch(
    requests: &MediaRequestExecutor,
    input: FetchInput<'_>,
    network: &crate::delivery_events::DeliveryNetworkStatusReader,
    mut cancellation: Option<tokio::sync::oneshot::Receiver<()>>,
) -> core::result::Result<FetchedObject, FetchFailure> {
    let spec = input.spec;
    let deadline = Instant::now() + spec.timeouts.total;
    let progress = FetchProgress::new(input.traffic);
    let runtime = FetchRuntime::new(requests, deadline, network, &progress);
    fetch_tracked(runtime, spec, &mut cancellation).await
}
