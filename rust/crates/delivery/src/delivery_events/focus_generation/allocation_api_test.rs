use super::{FocusGeneration, FocusGenerationGuard};
use core::num::NonZeroU64;

impl FocusGenerationGuard {
    pub(in crate::delivery_events) fn allocate(&mut self) -> Option<FocusGeneration> {
        let value = match self.latest {
            Some(latest) => latest.get().checked_add(1)?,
            None => 1,
        };
        let generated = NonZeroU64::new(value)?;
        self.latest = Some(generated);
        Some(FocusGeneration {
            value: Some(generated),
            covers_from: Some(generated),
        })
    }
}
