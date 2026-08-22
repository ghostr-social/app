use super::test_fixture::TransformFixture;
use super::{TransformJobs, TransformTerminal};
use crate::manager::transfers::InternalEvent;
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile,
};
use ghostr_engine::adaptive::TransformKind;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Default)]
struct DeadlineBackend {
    active: AtomicUsize,
    maximum: AtomicUsize,
    calls: AtomicUsize,
}

impl TransformBackend for DeadlineBackend {
    fn profile(&self) -> TransformProfile {
        let limits = TransformLimits::try_new(16, 16, 5, 5).unwrap();
        TransformProfile::new(TransformKind::Remux, limits)
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum.fetch_max(active, Ordering::SeqCst);
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        let result = if call == 0 {
            busy_until_cancelled(input, control)
        } else {
            TransformOutput::try_new(input.bytes().to_vec())
        };
        self.active.fetch_sub(1, Ordering::SeqCst);
        result
    }
}

fn busy_until_cancelled(
    input: TransformInput<'_>,
    control: &TransformControl,
) -> anyhow::Result<TransformOutput> {
    let started = Instant::now();
    let mut value = 1_u64;
    while started.elapsed() < Duration::from_millis(40) {
        value = black_box(value.wrapping_mul(31).wrapping_add(7));
    }
    black_box(value);
    control.checkpoint()?;
    TransformOutput::try_new(input.bytes().to_vec())
}

#[tokio::test]
async fn deadline_joins_worker_before_releasing_global_singleflight() {
    let fixture = TransformFixture::seeded("transform-deadline-owner").await;
    let backend = Arc::new(DeadlineBackend::default());
    let (events, mut receiver) = mpsc::unbounded_channel::<InternalEvent>();
    let resources = super::resource_test_fixture::control();
    let mut jobs = TransformJobs::new(Some(backend.clone()), events, resources);
    assert!(jobs.launch(fixture.store.clone(), fixture.request(10)));

    let InternalEvent::Transform(first) = receiver.recv().await.unwrap() else {
        panic!("expected Transform completion");
    };
    assert!(matches!(
        first.terminal,
        TransformTerminal::Failed("warp_transform_deadline_exceeded")
    ));
    assert!(first.actual_resources.unwrap().cpu_ms() > 0);
    assert!(jobs.finish(&first).is_some());
    assert!(jobs.launch(fixture.store.clone(), fixture.request(11)));
    let InternalEvent::Transform(second) = receiver.recv().await.unwrap() else {
        panic!("expected second Transform completion");
    };
    assert!(jobs.finish(&second).is_some());
    assert_eq!(backend.maximum.load(Ordering::SeqCst), 1);
}
