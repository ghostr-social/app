use super::delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use super::delivery_fixture::evidence::DeliveryEvidence as _;
use super::delivery_fixture::DeliveryHarness;
use super::{TAIL, TOTAL};
use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryHandle, FocusItem};
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::{RepresentationBinding, SourceGeneration, TransferIdentity};

pub(super) async fn next_request(origin: &mut ControlledOrigin, label: &str) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(5), origin.next())
        .await
        .unwrap_or_else(|_| panic!("{label} request starts"))
}

pub(super) async fn wait_closed(request: &ActiveRequest) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while request.is_open() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("receding zero-byte prefix is cancelled");
}

pub(super) async fn seed_tail(harness: &DeliveryHarness, item: &FocusItem) {
    let (binding, identity, generation) = tail_generation(item);
    harness
        .store
        .bind_representation(binding)
        .await
        .expect("binding");
    harness
        .store
        .select_transfer(identity.clone())
        .await
        .expect("selection");
    harness
        .store
        .accept_generation(&identity, generation.clone())
        .await
        .expect("generation");
    let bytes = vec![7; TAIL.len() as usize];
    let written = harness
        .store
        .write_range_for_generation_if_current(&identity, &generation, TAIL.start, &bytes)
        .await
        .expect("tail write");
    assert!(written, "completed tail uses the active generation");
}

fn tail_generation(
    item: &FocusItem,
) -> (RepresentationBinding, TransferIdentity, SourceGeneration) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(item.post.clone(), item.meta.clone());
    let source = item.meta.urls.first().expect("fixture source");
    let identity = binding.transfer(source).expect("fixture identity");
    let generation = SourceGeneration::try_new(source, "\"fixture-concurrency\"", TOTAL)
        .expect("fixture generation");
    (binding, identity, generation)
}

pub(super) fn pending_transfer_sequence(handle: &DeliveryHandle) -> u64 {
    let history = handle.decision_history();
    let pending = history.records.iter().filter(|record| {
        record.chosen_action_id.is_some() && record.eventual_outcome == DecisionOutcome::Pending
    });
    pending
        .max_by_key(|record| record.sequence)
        .expect("bound prefix transfer")
        .sequence
}
