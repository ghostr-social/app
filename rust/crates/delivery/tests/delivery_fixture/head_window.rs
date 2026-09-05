//! A visible current item that proves bootstrap bytes bypass advisory HEAD.

use super::items::{focus_now, unsized_item};
use super::media::{hit_log, hits, media_body, serve_recording, HitLog};
use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryHandle, FocusItem};
use std::sync::Arc;

pub struct VisibleCurrent {
    item: FocusItem,
    log: HitLog,
}

pub async fn serve_visible_current() -> VisibleCurrent {
    let log = hit_log();
    let url = serve_recording("current", media_body(), Arc::clone(&log)).await;
    VisibleCurrent {
        item: unsized_item("current", &url),
        log,
    }
}

impl VisibleCurrent {
    pub fn item(&self) -> FocusItem {
        self.item.clone()
    }

    pub async fn establish(&self, handle: &DeliveryHandle) {
        handle.update_focus(focus_now(vec![self.item()], 0, 0));
        self.assert_get_without_head().await;
    }

    pub async fn assert_get_without_head(&self) {
        tokio::time::timeout(Duration::from_secs(2), async {
            while !hits(&self.log)
                .iter()
                .any(|hit| hit.starts_with("current:GET:"))
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("visible current GET");
        assert!(
            hits(&self.log).iter().all(|hit| !hit.contains(":HEAD:")),
            "visible playback bypasses HEAD"
        );
    }
}
