//! A cancelled delivery receiver stops its watcher after the current error.

use crate::api::delivery_events_stream::{watch_delivery, EventOut};
use crate::api::delivery_types::{FfiDeliveryEvent, FfiDeliveryEventKind};
use crate::api::tests::support::{sized_meta, temp_store};
use crate::api::runtime::tracked_items::TrackedItems;
use std::sync::{Arc, Mutex};

struct RejectingOut(Arc<Mutex<Option<FfiDeliveryEvent>>>);

impl EventOut for RejectingOut {
    fn send(&self, event: FfiDeliveryEvent) -> bool {
        *self.0.lock().expect("event capture") = Some(event);
        false
    }
}

#[tokio::test]
async fn a_closed_receiver_ends_after_an_invalid_store_key_error() {
    let tracked = TrackedItems::new();
    tracked.insert("../unsafe".to_owned(), sized_meta(16, 2_000));
    let captured = Arc::new(Mutex::new(None));

    watch_delivery(
        RejectingOut(captured.clone()),
        temp_store("ghostr-api-cancelled-watch"),
        tracked,
    )
    .await;

    let event = captured
        .lock()
        .expect("event capture")
        .clone()
        .expect("error");
    assert_eq!(event.kind, FfiDeliveryEventKind::Error);
    assert!(event.detail.expect("error detail").contains("store keys"));
}
