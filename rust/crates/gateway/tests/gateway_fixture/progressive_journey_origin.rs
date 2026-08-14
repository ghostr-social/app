use sha2::{Digest, Sha256};

mod fixture;
mod fixture_expansion;
mod fixture_tail;
mod fixture_timing;
mod requests;
mod response;
mod server;

pub use requests::OriginRequest;
use server::{HeadBehavior, OriginState};

pub struct ProgressiveJourneyOrigin {
    pub url: String,
    state: OriginState,
    server: tokio::task::JoinHandle<()>,
}

impl ProgressiveJourneyOrigin {
    pub async fn with_blocked_head() -> Self {
        Self::start(fixture::progressive_mp4(), HeadBehavior::Blocked).await
    }

    pub async fn with_rejected_head() -> Self {
        Self::start(fixture::progressive_mp4(), HeadBehavior::Rejected).await
    }

    pub async fn with_lengthless_head() -> Self {
        Self::start(fixture::progressive_mp4(), HeadBehavior::Lengthless).await
    }

    pub async fn with_range_opaque_head() -> Self {
        Self::start(fixture::progressive_mp4(), HeadBehavior::RangeOpaque).await
    }

    pub async fn with_deferred_probe_and_failed_body() -> Self {
        Self::start(fixture::progressive_mp4(), HeadBehavior::DeferredFailure).await
    }

    pub async fn tail_moov_with_blocked_head() -> Self {
        Self::start(fixture::tail_moov_mp4(), HeadBehavior::Blocked).await
    }

    async fn start(bytes: Vec<u8>, head: HeadBehavior) -> Self {
        let running = server::start(bytes, head).await;
        Self {
            url: running.url,
            state: running.state,
            server: running.task,
        }
    }

    pub fn total_bytes(&self) -> u64 {
        self.state.bytes.len() as u64
    }

    pub fn sha256(&self) -> String {
        format!("{:x}", Sha256::digest(self.state.bytes.as_slice()))
    }

    pub fn requests(&self) -> Vec<OriginRequest> {
        self.state.requests.snapshot()
    }

    pub fn get_ranges(&self) -> Vec<std::ops::Range<u64>> {
        self.state.requests.get_ranges()
    }
}

impl Drop for ProgressiveJourneyOrigin {
    fn drop(&mut self) {
        self.server.abort();
    }
}
