use super::{HlsSessionId, HlsSessions, Instant, Url};

impl HlsSessions {
    pub async fn sources(&self, id: &HlsSessionId) -> Option<Vec<Url>> {
        let now = Instant::now();
        let mut state = self.state.lock().await;
        let session = state.active_session(id, now, self.limits.idle_ttl)?;
        Some(session.sources.clone())
    }
}
