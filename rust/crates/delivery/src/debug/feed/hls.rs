use super::{DebugFeed, DebugFeedItem};
use ghostr_engine::DeliveryKind;

impl DebugFeed {
    pub fn hls_items(&self) -> Vec<DebugFeedItem> {
        self.read()
            .items
            .iter()
            .filter(|item| item.meta.delivery == DeliveryKind::Hls)
            .cloned()
            .collect()
    }

    pub fn hls_sources(&self, id: &str) -> Option<Vec<String>> {
        self.read()
            .items
            .iter()
            .find(|item| item.id == id && item.meta.delivery == DeliveryKind::Hls)
            .map(|item| item.meta.urls.clone())
    }
}
