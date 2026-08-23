use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::prepare::{prepare_complete, PreparedComplete, PreparedObject};
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;
use std::sync::Arc;

const ROOT: &str = "https://media.example/root.m3u8";
const CHILD: &str = "https://media.example/child.m3u8";

#[tokio::test]
async fn completing_a_claimed_prefix_preserves_later_staged_objects() {
    let post = PostId::new("current");
    let cache = SegmentedCache::new();
    cache.replace_focus(1, vec![(post.clone(), vec![ROOT.to_owned()])]);
    let prefix = admission(
        &post,
        1,
        StageRequest::new(ROOT.to_owned(), 0, 64),
        StageReservation::block(64),
    );
    assert!(cache
        .admit_stage(prefix)
        .unwrap()
        .commit_partial(object(ROOT, 64)));
    let child = PreparedComplete::new(object(CHILD, 16));
    assert!(cache
        .admit_stage(child_admission(&post))
        .unwrap()
        .commit_complete(child));

    let final_stage = admission(
        &post,
        3,
        StageRequest::new(ROOT.to_owned(), 64, 32),
        StageReservation::final_block(32, 96).unwrap(),
    );
    let mut lease = cache.admit_stage(final_stage).unwrap();
    let block = object(ROOT, 32);
    let seed = lease.claim_assembly(&block).unwrap();
    let (_cancel, mut cancelled) = tokio::sync::oneshot::channel();
    let complete = prepare_complete(Some(seed), block, &mut cancelled)
        .await
        .unwrap();
    assert!(lease.commit_complete(complete));
    assert!(cache.mark_stage_ready(&post, 1));

    assert!(cache.object(ROOT).is_some());
    assert!(cache.object(CHILD).is_some());
}

fn child_admission(post: &PostId) -> StageAdmission {
    admission(
        post,
        2,
        StageRequest::new(CHILD.to_owned(), 0, 16),
        StageReservation::block(16),
    )
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

fn object(url: &str, bytes: usize) -> PreparedObject {
    PreparedObject {
        request_url: url.to_owned(),
        final_url: url.parse().unwrap(),
        body: Arc::from(vec![1; bytes]),
        content_type: None,
        cache: Default::default(),
    }
}
