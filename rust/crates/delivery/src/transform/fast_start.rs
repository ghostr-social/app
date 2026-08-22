use super::{
    TransformBackend, TransformControl, TransformInput, TransformLimits, TransformOutput,
    TransformProfile, TransformTrigger,
};
use anyhow::{ensure, Result};
use ghostr_engine::adaptive::TransformKind;

mod mp4;

pub struct FastStartRemuxBackend {
    profile: TransformProfile,
}

impl FastStartRemuxBackend {
    pub fn production() -> Self {
        let limits = TransformLimits::try_new(64 << 20, 64 << 20, 250, 1_000)
            .expect("production transform limits");
        Self {
            profile: TransformProfile::new(TransformKind::Remux, limits)
                .with_trigger(TransformTrigger::FastStartInvalidVideoTrack),
        }
    }
}

impl TransformBackend for FastStartRemuxBackend {
    fn profile(&self) -> TransformProfile {
        self.profile
    }

    fn transform(
        &self,
        input: TransformInput<'_>,
        control: &TransformControl,
    ) -> Result<TransformOutput> {
        ensure!(
            input.kind() == self.profile.kind(),
            "unsupported transform kind"
        );
        ensure!(
            input.bytes().len() as u64 <= self.profile.limits().input_bytes(),
            "transform input exceeds its byte envelope"
        );
        let output = mp4::fast_start(input.bytes(), control)?;
        ensure!(
            output.len() as u64 <= self.profile.limits().output_bytes(),
            "transform output exceeds its byte envelope"
        );
        TransformOutput::try_new(output)
    }
}
