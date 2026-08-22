use super::test_fixture::TransformFixture;
use super::TransformJobs;
use crate::manager::transfers::InternalEvent;
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile,
};
use ghostr_engine::adaptive::TransformKind;
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Default)]
struct CooperativeBackend {
    active: AtomicUsize,
    maximum: AtomicUsize,
    entered: AtomicBool,
}

impl TransformBackend for CooperativeBackend {
    fn profile(&self) -> TransformProfile {
        let limits = TransformLimits::try_new(16, 16, 100, 200).unwrap();
        TransformProfile::new(TransformKind::Remux, limits)
    }

    fn transform(
        &self,
        _input: TransformInput<'_>,
        control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum.fetch_max(active, Ordering::SeqCst);
        self.entered.store(true, Ordering::Release);
        let mut value = 1_u64;
        let result = loop {
            value = black_box(value.wrapping_mul(31).wrapping_add(7));
            if let Err(error) = control.checkpoint() {
                break Err(error);
            }
        };
        self.active.fetch_sub(1, Ordering::SeqCst);
        result
    }
}

#[tokio::test]
async fn cancellation_drains_worker_before_releasing_global_singleflight() {
    let fixture = TransformFixture::seeded("transform-cancel-owner").await;
    let backend = Arc::new(CooperativeBackend::default());
    let (events, mut receiver) = mpsc::unbounded_channel::<InternalEvent>();
    let resources = super::resource_test_fixture::control();
    let mut jobs = TransformJobs::new(Some(backend.clone()), events, resources);
    assert!(jobs.launch(fixture.store.clone(), fixture.request(20)));
    wait_until_entered(&backend).await;

    assert_eq!(jobs.clear(), 1);
    assert!(!jobs.launch(fixture.store.clone(), fixture.request(21)));
    let InternalEvent::Transform(first) = receiver.recv().await.unwrap() else {
        panic!("expected cancelled Transform completion");
    };
    assert!(first.actual_resources.is_some());
    assert!(jobs.finish(&first).is_some());
    assert!(jobs.launch(fixture.store.clone(), fixture.request(21)));
    jobs.clear();
    let InternalEvent::Transform(second) = receiver.recv().await.unwrap() else {
        panic!("expected second Transform completion");
    };
    assert!(jobs.finish(&second).is_some());
    assert_eq!(backend.maximum.load(Ordering::SeqCst), 1);
}

async fn wait_until_entered(backend: &CooperativeBackend) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while !backend.entered.load(Ordering::Acquire) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}
