use super::prepare_transfer;
use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::fetch::{FetchedObject, OriginTelemetry};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::PostId;
use std::future::{poll_fn, Future};
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

const KIB: u64 = 1024;
const URL: &str = "https://media.example/segment.m4s";

#[tokio::test]
async fn supervised_preparation_cancels_without_publishing_assembled_bytes() {
    let cache = SegmentedCache::new();
    let post = PostId::new("current");
    cache.replace_focus(1, vec![(post.clone(), vec![URL.to_owned()])]);
    let prefix_request = StageRequest::new(URL.to_owned(), 0, 1024 * KIB);
    let prefix_fence = StageFence::new(1, 1, prefix_request);
    let prefix = StageAdmission::new(post.clone(), prefix_fence, 500, (1024 * KIB).into());
    assert!(cache
        .admit_stage(prefix)
        .unwrap()
        .commit_partial(prepared((1024 * KIB) as usize)));
    let request = StageRequest::new(URL.to_owned(), 1024 * KIB, 128 * KIB);
    let fence = StageFence::new(1, 7, request);
    let reservation = StageReservation::final_block(128 * KIB, 1152 * KIB).unwrap();
    let lease = cache
        .admit_stage(StageAdmission::new(post, fence, 500, reservation))
        .unwrap();
    let (cancel, cancelled) = tokio::sync::oneshot::channel();
    let future = prepare_transfer(lease, fetched(), cancelled);
    tokio::pin!(future);

    poll_fn(|context| match future.as_mut().poll(context) {
        Poll::Pending => Poll::Ready(()),
        Poll::Ready(_) => panic!("preparation crossed its cancellation checkpoint"),
    })
    .await;
    assert_eq!(cache.physical_used_bytes(), 2304 * KIB);
    cancel.send(()).unwrap();
    let failure = match future.await {
        Ok(_) => panic!("cancelled preparation must not publish"),
        Err(failure) => failure,
    };

    assert!(failure.is_cancelled());
    assert!(failure.response_completed());
    assert_eq!(failure.network_bytes(), 128 * KIB);
    assert_eq!(cache.physical_used_bytes(), 1024 * KIB);
}

fn fetched() -> FetchedObject {
    FetchedObject {
        request_url: URL.to_owned(),
        final_url: URL.parse().unwrap(),
        body: Arc::from(vec![8; (128 * KIB) as usize]),
        content_type: Some("video/mp4".to_owned()),
        cache: Default::default(),
        telemetry: OriginTelemetry {
            elapsed: Duration::from_millis(25),
            ttfb: Some(Duration::from_millis(5)),
            concurrency: 1,
            network_class: NetworkClass::Wifi,
        },
        offset: 1024 * KIB,
        continuation: None,
    }
}

fn prepared(bytes: usize) -> PreparedObject {
    PreparedObject {
        request_url: URL.to_owned(),
        final_url: URL.parse().unwrap(),
        body: Arc::from(vec![7; bytes]),
        content_type: Some("video/mp4".to_owned()),
        cache: Default::default(),
    }
}
