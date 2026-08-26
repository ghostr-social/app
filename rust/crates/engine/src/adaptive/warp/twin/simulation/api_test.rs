use super::DigitalTwin;

impl DigitalTwin {
    pub(crate) fn cache_entries(&self) -> usize {
        self.cache.len()
    }
}
