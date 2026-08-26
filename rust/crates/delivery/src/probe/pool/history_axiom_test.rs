use super::*;

use ghostr_engine::representation::HttpGenerationLease;

impl CompletedHeadProbe {
    pub(in super::super) fn for_test(generation: Option<HttpGenerationLease>) -> Self {
        Self {
            stamp: generation.map(HttpGenerationStamp::from_trusted),
            observed_size: true,
        }
    }
}
