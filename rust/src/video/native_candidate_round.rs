use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub(crate) struct NativeCandidateRound {
    attempted: HashSet<String>,
    retryable: bool,
}

impl NativeCandidateRound {
    pub(crate) fn is_pending(&self, url: &str) -> bool {
        !self.attempted.contains(url)
    }

    pub(crate) fn record_failure(&mut self, url: &str, retryable: bool) {
        self.attempted.insert(url.to_owned());
        self.retryable |= retryable;
    }

    pub(crate) fn is_retryable(&self) -> bool {
        self.retryable
    }

    pub(crate) fn reset(&mut self) {
        self.attempted.clear();
        self.retryable = false;
    }
}
