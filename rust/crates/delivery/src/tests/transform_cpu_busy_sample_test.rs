use super::test_fixture::TransformFixture;
use super::TransformJobs;
use crate::manager::transfers::InternalEvent;
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile,
};
use ghostr_engine::adaptive::TransformKind;
use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

struct BusyBackend;

impl TransformBackend for BusyBackend {
    fn profile(&self) -> TransformProfile {
        let limits = TransformLimits::try_new(16, 16, 100, 200).unwrap();
        TransformProfile::new(TransformKind::Remux, limits)
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        let started = Instant::now();
        let mut value = 1_u64;
        while started.elapsed() < Duration::from_millis(20) {
            value = black_box(value.wrapping_mul(31).wrapping_add(7));
            control.checkpoint()?;
        }
        black_box(value);
        TransformOutput::try_new(input.bytes().to_vec())
    }
}

#[tokio::test]
async fn bounded_busy_transform_records_consumed_cpu() {
    let fixture = TransformFixture::seeded("transform-cpu-busy").await;
    let (events, mut receiver) = mpsc::unbounded_channel::<InternalEvent>();
    let mut jobs = TransformJobs::new(Some(Arc::new(BusyBackend)), events);
    assert!(jobs.launch(fixture.store.clone(), fixture.request(8)));

    let InternalEvent::Transform(done) = receiver.recv().await.unwrap() else {
        panic!("expected Transform completion");
    };
    let actual = done.actual_resources.expect("measured resources");
    assert!(actual.cpu_ms() > 0, "busy work must consume CPU");
    assert!(jobs.finish(&done).is_some());
    assert_eq!(jobs.take_cpu_sample_ms(), Some(actual.cpu_ms()));
}
