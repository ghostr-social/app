use core::time::Duration;
use ghostr_delivery::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile, TransformTrigger,
};
use ghostr_engine::adaptive::TransformKind;
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Default)]
pub struct BlockingRemux {
    entered: Arc<Notify>,
}

impl BlockingRemux {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn wait_until_entered(&self) {
        tokio::time::timeout(Duration::from_secs(2), self.entered.notified())
            .await
            .expect("valid test fixture");
    }
}

impl TransformBackend for BlockingRemux {
    fn profile(&self) -> TransformProfile {
        let limits =
            TransformLimits::try_new(1_024, 1_024, 500, 1_000).expect("valid test fixture");
        TransformProfile::new(TransformKind::Remux, limits)
            .with_trigger(TransformTrigger::InvalidVideoTrack)
    }

    fn transform(
        &self,
        _input: TransformInput<'_>,
        control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        self.entered.notify_one();
        loop {
            control.checkpoint()?;
            std::thread::yield_now();
        }
    }
}
