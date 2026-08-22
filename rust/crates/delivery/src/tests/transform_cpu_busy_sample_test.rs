use super::test_fixture::TransformFixture;
use super::TransformJobs;
use crate::manager::resource_control::{ResourceControl, ResourceEnvironment};
use crate::manager::transfers::InternalEvent;
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile,
};
use ghostr_engine::adaptive::ResourceObservation;
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

#[tokio::test(start_paused = true)]
async fn bounded_busy_transform_charges_cpu_before_manager_completion() {
    let fixture = TransformFixture::seeded("transform-cpu-busy").await;
    let (events, mut receiver) = mpsc::unbounded_channel::<InternalEvent>();
    let target = ResourceObservation::new(1, 1, 100, 1);
    let environment = ResourceEnvironment::new(0, target);
    let resources = ResourceControl::new(tokio::time::Instant::now(), environment);
    let mut jobs = TransformJobs::new(Some(Arc::new(BusyBackend)), events, resources.clone());
    assert!(jobs.launch(fixture.store.clone(), fixture.request(8)));

    let InternalEvent::Transform(done) = receiver.recv().await.unwrap() else {
        panic!("expected Transform completion");
    };
    let actual = done.actual_resources.expect("measured resources");
    assert!(actual.cpu_ms() > 0, "busy work must consume CPU");
    tokio::time::advance(Duration::from_millis(500)).await;
    let feedback = resources.feedback(environment);
    assert_eq!(feedback.actual.cpu, actual.cpu_ms());
    assert!(jobs.finish(&done).is_some());
}
