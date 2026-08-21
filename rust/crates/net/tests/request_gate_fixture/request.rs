use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};

pub async fn open(
    requests: MediaRequestExecutor,
    url: String,
    priority: PreemptionAuthority,
) -> MediaResponse {
    requests
        .get(&url, priority)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await
        .unwrap()
}
