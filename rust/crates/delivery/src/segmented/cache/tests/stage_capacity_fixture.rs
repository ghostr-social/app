use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::prepare::{PreparedComplete, PreparedObject};
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;
use std::sync::Arc;

pub(super) const MIB: usize = 1024 * 1024;

pub(super) fn cache_with_ready_bytes(
    ready_bytes: usize,
    prefix_bytes: usize,
) -> (SegmentedCache, PostId) {
    let cache = SegmentedCache::new();
    let held = PostId::new("held");
    let current = PostId::new("current");
    cache.replace_focus(
        1,
        vec![
            (held.clone(), vec![url("held")]),
            (current.clone(), vec![url("current")]),
        ],
    );
    store_complete(&cache, &held, object("held", ready_bytes));
    assert!(cache.mark_stage_ready(&held, 1));
    store_partial(&cache, &current, prefix_bytes);
    (cache, current)
}

pub(super) fn final_admission(
    post: &PostId,
    prefix_bytes: usize,
    block_bytes: usize,
    total_bytes: usize,
) -> StageAdmission {
    let request = StageRequest::new(url("current"), prefix_bytes as u64, block_bytes as u64);
    let reservation =
        StageReservation::final_block(block_bytes as u64, total_bytes as u64).unwrap();
    admission(post, 3, request, reservation)
}

pub(super) fn object(name: &str, bytes: usize) -> PreparedObject {
    let url = url(name);
    PreparedObject {
        request_url: url.clone(),
        final_url: url.parse().unwrap(),
        body: Arc::from(vec![0; bytes]),
        content_type: None,
        cache: Default::default(),
    }
}

fn store_complete(cache: &SegmentedCache, post: &PostId, object: PreparedObject) {
    let bytes = object.body.len() as u64;
    let request = StageRequest::new(object.request_url.clone(), 0, bytes);
    let lease = cache
        .admit_stage(admission(post, 1, request, StageReservation::block(bytes)))
        .unwrap();
    assert!(lease.commit_complete(PreparedComplete::new(object)));
}

fn store_partial(cache: &SegmentedCache, post: &PostId, bytes: usize) {
    let request = StageRequest::new(url("current"), 0, bytes as u64);
    let lease = cache
        .admit_stage(admission(
            post,
            2,
            request,
            StageReservation::block(bytes as u64),
        ))
        .unwrap();
    assert!(lease.commit_partial(object("current", bytes)));
}

fn admission(
    post: &PostId,
    attempt: u64,
    request: StageRequest,
    reservation: StageReservation,
) -> StageAdmission {
    StageAdmission::new(
        post.clone(),
        StageFence::new(1, attempt, request),
        500,
        reservation,
    )
}

fn url(name: &str) -> String {
    format!("https://example.com/{name}")
}
