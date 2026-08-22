use crate::delivery_events::FocusItem;
use ghostr_engine::adaptive::FeedOffset;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{DeliveryKind, PostId};

#[derive(Clone, Eq, PartialEq)]
pub(super) struct Target {
    pub(super) post: PostId,
    pub(super) sources: Vec<String>,
    pub(super) priority: PreemptionAuthority,
    pub(super) offset: FeedOffset,
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
            offset: FeedOffset::new(offset.min(i32::MAX as usize) as i32),
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
