use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;
use std::sync::Arc;

pub(super) fn focused_cache(post: &PostId) -> SegmentedCache {
    let cache = SegmentedCache::new();
    cache.replace_focus(1, vec![(post.clone(), vec![source()])]);
    cache
}

pub(super) fn store_partial(cache: &SegmentedCache, post: &PostId, bytes: usize) {
    let request = StageRequest::new(source(), 0, bytes as u64);
    let fence = StageFence::new(1, 1, request);
    let admission = StageAdmission::new(
        post.clone(),
        fence,
        500,
        StageReservation::block(bytes as u64),
    );
    let lease = cache.admit_stage(admission).expect("valid test fixture");
    assert!(lease.commit_partial(object(bytes)));
}

pub(super) fn object(bytes: usize) -> PreparedObject {
    PreparedObject {
        request_url: source(),
        final_url: source().parse().expect("valid test fixture"),
        body: Arc::from(vec![7; bytes]),
        content_type: Some("video/mp4".to_owned()),
        cache: Default::default(),
    }
}

pub(super) fn source() -> String {
    "https://media.example/segment.m4s".to_owned()
}
