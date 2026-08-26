use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};

pub async fn open(
    requests: MediaRequestExecutor,
    url: String,
    priority: PreemptionAuthority,
) -> MediaResponse {
    requests
        .get(&url, priority)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await
        .expect("valid test fixture")
}
