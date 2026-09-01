use super::{stage_fence, stage_reservation};
use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::fetch::ObjectContinuation;
use crate::segmented::prepare::PreparedObject;
use crate::segmented::scheduler::progress::Pending;
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::PostId;
use ghostr_net::strong_etag::single_strong_etag;
use reqwest::header::{HeaderMap, HeaderValue, ETAG};
use std::sync::Arc;

const KIB: u64 = 1024;
const PREFIX: u64 = 1024 * KIB;
const BLOCK: u64 = 128 * KIB;
const TOTAL: u64 = PREFIX + BLOCK;
const URL: &str = "https://media.example/init.mp4";

#[test]
fn final_stage_admission_reserves_the_full_assembly_peak() {
    let cache = SegmentedCache::new();
    let post = PostId::new("stream");
    cache.replace_focus(1, vec![(post.clone(), vec![URL.to_owned()])]);
    let prefix = cache
        .admit_stage(admission(&post, 1, 0, PREFIX))
        .expect("valid test fixture");
    assert!(prefix.commit_partial(object(PREFIX as usize)));
    let pending = pending();
    let reservation = stage_reservation(&pending, BLOCK).expect("valid test fixture");
    let fence = stage_fence(&pending, BLOCK);

    let lease = cache
        .admit_stage(StageAdmission::new(post, fence, 500, reservation))
        .expect("valid test fixture");

    assert_eq!(cache.physical_used_bytes(), 2 * TOTAL);
    drop(lease);
}

fn pending() -> Pending {
    Pending {
        generation: 1,
        attempt: 2,
        generation_restarts: 0,
        source_index: 0,
        root_source: URL.to_owned(),
        playback_manifest: URL.to_owned(),
        stage: HlsBootstrapStage::Initialization,
        url: URL.to_owned(),
        after_init: None,
        continuation: Some(continuation()),
    }
}

fn continuation() -> ObjectContinuation {
    let mut headers = HeaderMap::new();
    headers.insert(ETAG, HeaderValue::from_static("\"v1\""));
    ObjectContinuation {
        next_offset: PREFIX,
        total: TOTAL,
        final_url: URL.parse().expect("valid test fixture"),
        strong_etag: single_strong_etag(&headers)
            .expect("valid test fixture")
            .expect("valid test fixture"),
    }
}

fn admission(post: &PostId, attempt: u64, offset: u64, bytes: u64) -> StageAdmission {
    let request = StageRequest::new(URL.to_owned(), offset, bytes);
    let fence = StageFence::new(1, attempt, request);
    StageAdmission::new(post.clone(), fence, 500, StageReservation::block(bytes))
}

fn object(bytes: usize) -> PreparedObject {
    PreparedObject {
        request_url: URL.to_owned(),
        final_url: URL.parse().expect("valid test fixture"),
        body: Arc::from(vec![7; bytes]),
        content_type: Some("video/mp4".to_owned()),
        cache: Default::default(),
    }
}
