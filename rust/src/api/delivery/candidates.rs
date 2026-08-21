use crate::discovery::content::candidates::VideoCandidate;
use crate::engine::{PostId, PreviewDescriptor};
use ghostr_delivery::delivery_events::{DeliveryCandidate, DeliveryHandle};

pub(crate) fn admit(handle: Option<&DeliveryHandle>, candidate: Option<VideoCandidate>) {
    let (Some(handle), Some(candidate)) = (handle, candidate) else {
        return;
    };
    handle.admit_candidate(delivery_candidate(candidate));
}

pub(crate) fn delivery_candidate(candidate: VideoCandidate) -> DeliveryCandidate {
    DeliveryCandidate {
        post: PostId::new(candidate.id.as_str()),
        meta: candidate.post.meta,
        preview: candidate
            .post
            .blurhash
            .as_deref()
            .and_then(PreviewDescriptor::inline_blurhash),
        metadata_evidence: candidate.post.metadata_evidence,
        renditions: candidate.post.renditions,
        discovered_at: candidate.post.feed_sort_at,
    }
}
