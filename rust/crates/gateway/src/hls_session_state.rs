use crate::hls_session_types::{random_id, random_secret, HlsSessionId};
use reqwest::Url;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::Instant;

#[derive(Default)]
pub(crate) struct HlsSessionState {
    pub sessions: HashMap<HlsSessionId, HlsSession>,
}

impl HlsSessionState {
    pub fn prune(&mut self, now: Instant, ttl: Duration) {
        self.sessions
            .retain(|_, session| now.duration_since(session.last_used) < ttl);
    }

    pub fn unique_id(&self) -> HlsSessionId {
        loop {
            let candidate = random_id();
            if !self.sessions.contains_key(&candidate) {
                return candidate;
            }
        }
    }

    pub fn active_session(
        &mut self,
        id: &HlsSessionId,
        now: Instant,
        ttl: Duration,
    ) -> Option<&mut HlsSession> {
        self.prune(now, ttl);
        let session = self.sessions.get_mut(id)?;
        session.last_used = now;
        Some(session)
    }
}

pub(crate) struct HlsSession {
    pub sources: Vec<Url>,
    pub last_used: Instant,
    pub secret: [u8; 32],
}

impl HlsSession {
    pub fn new(sources: Vec<Url>, now: Instant) -> Self {
        Self {
            sources,
            last_used: now,
            secret: random_secret(),
        }
    }
}
