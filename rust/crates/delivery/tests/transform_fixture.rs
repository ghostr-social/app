use ghostr_delivery::transform::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile, TransformTrigger,
};
use ghostr_engine::adaptive::TransformKind;
use std::time::Duration;

pub const INPUT: &[u8] = b"ftyp|mdat:frames|moov:index";
pub const OUTPUT: &[u8] = b"ftyp|moov:index|mdat:frames";

pub struct FixtureRemux;

impl TransformBackend for FixtureRemux {
    fn profile(&self) -> TransformProfile {
        let limits = TransformLimits::try_new(1_024, 1_024, 5, 250).unwrap();
        TransformProfile::new(TransformKind::Remux, limits)
            .with_trigger(TransformTrigger::InvalidVideoTrack)
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        control: &TransformControl,
    ) -> anyhow::Result<TransformOutput> {
        control.checkpoint()?;
        anyhow::ensure!(input.kind() == TransformKind::Remux, "wrong transform kind");
        anyhow::ensure!(input.bytes() == INPUT, "wrong transform input");
        std::thread::sleep(Duration::from_millis(2));
        control.checkpoint()?;
        TransformOutput::try_new(OUTPUT.to_vec())
    }
}
