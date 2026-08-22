use crate::delivery_fixture::items::{focus_now, sized_item};
use crate::delivery_fixture::options::DeliveryOptions;
use crate::delivery_fixture::{start_harness_with_store, temp_directory, DeliveryHarness};
use crate::focus_wait_fixture::wait_for_focus;
use crate::transform_delivery_fixture::{report_unsupported, seed_input};
use ghostr_delivery::transform::TransformBackend;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

pub const INPUT: &[u8] = b"ftyp|mdat:priced|moov:index";

pub async fn start(prefix: &str, backend: Arc<dyn TransformBackend>) -> DeliveryHarness {
    let root = temp_directory(prefix);
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let item = sized_item(
        "post",
        "https://origin.example/video.mp4",
        INPUT.len() as u64,
        1_000,
    );
    let input = seed_input(&store, &item, INPUT).await;
    let options = DeliveryOptions {
        transform: Some(backend),
        ..DeliveryOptions::default()
    };
    let harness = start_harness_with_store(store, root, options);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    wait_for_focus(&harness.cache).await;
    report_unsupported(&harness.handle, &harness.store, input).await;
    harness
}
