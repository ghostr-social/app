use crate::discovery::candidate_registry::VideoCandidate;
use crate::engine::PostId;
use ghostr_delivery::delivery_events::{DeliveryCandidate, DeliveryHandle};

pub(crate) fn admit(handle: Option<&DeliveryHandle>, candidate: Option<VideoCandidate>) {
    let (Some(handle), Some(candidate)) = (handle, candidate) else {
        return;
    };
    handle.admit_candidate(DeliveryCandidate {
        post: PostId::new(candidate.id.as_str()),
        meta: candidate.post.meta,
        discovered_at: candidate.post.created_at,
    });
}
