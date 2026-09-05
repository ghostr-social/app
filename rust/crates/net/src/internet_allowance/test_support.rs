use super::{InternetAllowance, InternetDataLimit, InternetUsage};

impl InternetAllowance {
    pub fn memory(limit: InternetDataLimit) -> Self {
        Self::from_state(limit, InternetUsage::default(), None)
    }

    pub fn usage(&self) -> (u64, u64) {
        let usage = self.lock().usage;
        (usage.charged_bytes, usage.reserved_bytes)
    }
}
