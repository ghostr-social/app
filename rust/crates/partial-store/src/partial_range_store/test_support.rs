use super::PartialRangeStore;

impl PartialRangeStore {
    /// Reports whether a body response is registered without exposing response internals.
    pub async fn response_open_for_test(&self, key: &str) -> bool {
        if self.single_response_actions.lock().await.contains_key(key) {
            return true;
        }
        self.sparse_response_actions
            .lock()
            .await
            .values()
            .any(|state| state.identity.post().as_str() == key)
    }
}
