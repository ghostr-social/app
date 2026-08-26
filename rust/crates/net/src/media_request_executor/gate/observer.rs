use super::super::MediaResourceObserver;
use super::RequestLease;
use std::sync::{Arc, OnceLock};

#[derive(Default)]
pub(super) struct ResourceObserverSlot {
    observer: OnceLock<Arc<dyn MediaResourceObserver>>,
}

impl ResourceObserverSlot {
    pub(super) fn install(&self, observer: Arc<dyn MediaResourceObserver>) -> bool {
        self.observer.set(observer).is_ok()
    }

    fn record_request(&self) {
        if let Some(observer) = self.get() {
            observer.record_request();
        }
    }

    fn record_response_bytes(&self, bytes: u64) {
        if let Some(observer) = self.get() {
            observer.record_response_bytes(bytes);
        }
    }

    fn get(&self) -> Option<Arc<dyn MediaResourceObserver>> {
        self.observer.get().cloned()
    }
}

impl RequestLease {
    pub(in crate::media_request_executor) fn record_request(&self) {
        self.gate.inner.observer.record_request();
    }

    pub(in crate::media_request_executor) fn record_response_bytes(&self, bytes: u64) {
        self.gate.inner.observer.record_response_bytes(bytes);
    }
}
