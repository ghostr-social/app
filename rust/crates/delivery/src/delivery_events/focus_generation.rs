use core::num::NonZeroU64;

#[cfg(test)]
mod allocation_api_test;

/// Monotonic identity of one user focus intent. Compatibility calls
/// are explicit and do not participate in version ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusGeneration {
    value: Option<NonZeroU64>,
    covers_from: Option<NonZeroU64>,
}

impl FocusGeneration {
    pub fn try_new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(|value| Self {
            value: Some(value),
            covers_from: Some(value),
        })
    }

    pub const fn compatibility() -> Self {
        Self {
            value: None,
            covers_from: None,
        }
    }

    pub const fn value(self) -> Option<u64> {
        match self.value {
            Some(value) => Some(value.get()),
            None => None,
        }
    }

    pub(crate) const fn covers_from_value(self) -> Option<u64> {
        match self.covers_from {
            Some(value) => Some(value.get()),
            None => None,
        }
    }

    pub(crate) fn covering(mut self, prior: Self) -> Self {
        let (Some(start), Some(prior_end), Some(prior_start)) =
            (self.covers_from, prior.value, prior.covers_from)
        else {
            return self;
        };
        if start.get() > prior_end.get().saturating_add(1) {
            return self;
        }
        self.covers_from = Some(start.min(prior_start));
        self
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
        let Some(candidate) = generation.value else {
            return true;
        };
        if self.latest.is_some_and(|latest| candidate <= latest) {
            return false;
        }
        self.latest = Some(candidate);
        true
    }
}
