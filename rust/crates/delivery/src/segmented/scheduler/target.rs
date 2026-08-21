use crate::delivery_events::FocusItem;
use crate::manager::transfers::InternalEvent;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{DeliveryKind, PostId};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Eq, PartialEq)]
pub(super) struct Target {
    pub(super) post: PostId,
    pub(super) sources: Vec<String>,
    pub(super) priority: PreemptionAuthority,
}

pub(crate) struct ReconcileInput {
    pub requests: MediaRequestExecutor,
    pub events: UnboundedSender<InternalEvent>,
    pub connection_limit: usize,
    pub progressive_active: usize,
}

pub(super) fn targets(items: &[FocusItem], current: usize, limit: usize) -> Vec<Target> {
    items[current..]
        .iter()
        .take(limit)
        .enumerate()
        .filter(|(_, item)| item.meta.delivery == DeliveryKind::Hls)
        .map(|(offset, item)| Target {
            post: item.post.clone(),
            sources: item.meta.urls.clone(),
            priority: priority(offset),
        })
        .collect()
}

fn priority(offset: usize) -> PreemptionAuthority {
    match offset {
        0 => PreemptionAuthority::PlaybackCritical,
        1 => PreemptionAuthority::Transition,
        _ => PreemptionAuthority::Speculative,
    }
}
