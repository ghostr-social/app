use super::progress::Pending;
use super::{active_network, test_fence, Active};
use crate::segmented::cache::{
    StageAdmission, StageFence, StageLease, StageRequest, StageReservation,
};
use crate::segmented::prepare::{PreparedComplete, PreparedObject};
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::{ActionId, PostId};

pub(super) fn active(post: &str) -> Active {
    let (cancellation, _cancelled) = tokio::sync::oneshot::channel();
    let pending = pending(post);
    Active {
        action: ActionId::new(1),
        fence: test_fence(
            1,
            1,
            &pending.url,
            HlsBootstrapStage::Initialization.maximum_bytes(),
        ),
        pending,
        committed_until_ms: u64::MAX,
        network: active_network(),
        _task: tokio::spawn(core::future::pending()),
        cancellation: Some(cancellation),
        cancelling: false,
    }
}

pub(super) fn pending(post: &str) -> Pending {
    Pending {
        generation: 1,
        attempt: 1,
        generation_restarts: 0,
        source_index: 0,
        root_source: format!("https://{post}.example/root.m3u8"),
        stage: HlsBootstrapStage::Initialization,
        url: format!("https://{post}.example/init.mp4"),
        after_init: Some(format!("https://{post}.example/first.m4s")),
        continuation: None,
    }
}

pub(super) fn reserve_active(cache: &SegmentedCache, post: &PostId, active: &Active) -> StageLease {
    let bytes = HlsBootstrapStage::Initialization.maximum_bytes();
    let admission = StageAdmission::new(
        post.clone(),
        active.fence.clone(),
        500,
        StageReservation::block(bytes),
    );
    cache.admit_stage(admission).expect("active stage reserved")
}

pub(super) fn store_complete(
    cache: &SegmentedCache,
    post: &PostId,
    attempt: u64,
    object: PreparedObject,
) {
    let bytes = object.body.len() as u64;
    let request = StageRequest::new(object.request_url.clone(), 0, bytes);
    let fence = StageFence::new(1, attempt, request);
    let admission = StageAdmission::new(post.clone(), fence, 500, StageReservation::block(bytes));
    let lease = cache.admit_stage(admission).expect("ready object reserved");
    assert!(lease.commit_complete(PreparedComplete::new(object)));
}
