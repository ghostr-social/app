use std::num::NonZeroU64;

/// Monotonic identity of one user focus intent. Compatibility calls
/// are explicit and do not participate in version ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusGeneration(Option<NonZeroU64>);

impl FocusGeneration {
    pub fn try_new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(|value| Self(Some(value)))
    }

    pub const fn compatibility() -> Self {
        Self(None)
    }

    pub const fn value(self) -> Option<u64> {
        match self.0 {
            Some(value) => Some(value.get()),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusAdmission {
    Accepted,
    Stale,
    Closed,
}

#[derive(Debug, Default)]
pub(crate) struct FocusGenerationGuard {
    latest: Option<NonZeroU64>,
}

impl FocusGenerationGuard {
    pub(crate) fn accept(&mut self, generation: FocusGeneration) -> bool {
        let Some(candidate) = generation.0 else {
            return true;
        };
        if self.latest.is_some_and(|latest| candidate <= latest) {
            return false;
        }
        self.latest = Some(candidate);
        true
    }
}
