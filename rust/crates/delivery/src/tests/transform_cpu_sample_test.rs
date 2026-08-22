use super::test_fixture::TransformFixture;
use super::TransformJobs;
use crate::manager::transfers::InternalEvent;
use crate::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile,
};
use ghostr_engine::adaptive::TransformKind;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

struct SleepingBackend;

impl TransformBackend for SleepingBackend {
    fn profile(&self) -> TransformProfile {
        TransformProfile::new(
            TransformKind::Remux,
            TransformLimits::try_new(16, 16, 20, 5_000).unwrap(),
        )
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        _control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        std::thread::sleep(Duration::from_millis(20));
        TransformOutput::try_new(input.bytes().to_vec())
    }
}

#[tokio::test]
async fn sleeping_transform_records_zero_cpu() {
    let fixture = TransformFixture::seeded("transform-cpu-sleep").await;
    let (events, mut receiver) = mpsc::unbounded_channel::<InternalEvent>();
    let resources = super::resource_test_fixture::control();
    let mut jobs = TransformJobs::new(Some(Arc::new(SleepingBackend)), events, resources);
    assert!(jobs.launch(fixture.store.clone(), fixture.request(7)));

    let InternalEvent::Transform(done) = receiver.recv().await.unwrap() else {
        panic!("expected Transform completion");
    };
    assert!(matches!(
        done.terminal,
        super::TransformTerminal::Succeeded(4)
    ));
    let actual = done.actual_resources.expect("measured resources");
    assert_eq!(actual.cpu_ms(), 0, "sleep is not CPU consumption");
    assert_eq!(actual.storage_bytes(), 4);
    assert!(jobs.finish(&done).is_some());
}
