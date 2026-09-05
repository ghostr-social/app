use super::{Arc, InternetAllowance, MediaHttpRequests, MediaRequestExecutor, MediaRequestLimits};
use crate::internet_allowance::InternetDataLimit;

impl MediaRequestExecutor {
    pub fn new(client: Arc<dyn MediaHttpRequests>, limits: MediaRequestLimits) -> Self {
        Self::with_allowance(
            client,
            limits,
            InternetAllowance::memory(InternetDataLimit::Unlimited),
        )
    }
}
